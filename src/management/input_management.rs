use std::io;
use std::time::Duration;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, MouseButton, MouseEvent, MouseEventKind};

use crate::app::{ActionKind, App};
use crate::lsp::{LspMessage, LspStatus};

impl App {
    pub fn handle_events(&mut self) -> io::Result<()> {
        if self.poll_and_dispatch(Duration::from_millis(16))? {
            while self.poll_and_dispatch(Duration::from_millis(0))? {}
        }

        self.poll_hover();
        self.poll_semantic_tokens();

        let current_language = self.language_id();
        let languages: Vec<String> = self.lsp_sessions.keys().cloned().collect();

        for language in languages {
            let is_current = current_language == Some(language.as_str());
            loop {
                let message = match self.lsp_sessions.get(&language) {
                    Some(session) => session.client.try_recv(),
                    None => None,
                };
                let Some(message) = message else { break; };

                match message {
                    LspMessage::Initialized(legend) => {
                        if let Some(session) = self.lsp_sessions.get_mut(&language) {
                            let _ = session.client.initialized();
                            session.semantic_legend = legend;
                            session.status = LspStatus::Alive;
                        }
                        if is_current { self.open_current_document(); }
                    }
                    LspMessage::Completion(mut items) if is_current => {
                        let prefix = self.document.lines[self.cursor.y][..self.cursor.x].rsplit(|character: char| !character.is_alphanumeric() && character != '_').next().unwrap_or_default().to_lowercase();
                        if !prefix.is_empty() { items.retain(|item| item.label.to_lowercase().starts_with(&prefix)); }
                        self.completions = items;
                        self.completion_selected = 0;
                    }
                    LspMessage::Diagnostics(uri, items) if is_current && uri == crate::lsp::protocol::path_to_uri(std::path::Path::new(&self.current_file)) => {
                        self.diagnostics = items;
                    }
                    LspMessage::Hover(line, character, hover) if is_current && self.hover_position == Some((line, character)) => {
                        self.hover = hover;
                    }
                    LspMessage::SemanticTokens(data) if is_current => {
                        let legend = self.lsp_sessions.get(&language).map(|s| s.semantic_legend.clone()).unwrap_or_default();
                        self.set_semantic_tokens(&legend, data);
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    /// Polls for a single terminal event within `timeout` and dispatches it.
    /// Returns Ok(true) if an event was found and handled.
    fn poll_and_dispatch(&mut self, timeout: Duration) -> io::Result<bool> {
        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    self.handle_key(key);
                }
                Event::Mouse(mouse) => {
                    self.handle_mouse(mouse);
                }
                _ => {}
            }
            return Ok(true);
        }
        Ok(false)
    }

    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) {
        if self.status {
            self.reset_status();
        }
        if !self.completions.is_empty() { match key.code { KeyCode::Esc => { self.completions.clear(); return; }, KeyCode::Up => { self.completion_selected = self.completion_selected.saturating_sub(1); return; }, KeyCode::Down => { self.completion_selected = (self.completion_selected + 1).min(self.completions.len() - 1); return; }, KeyCode::Enter | KeyCode::Tab => { self.accept_completion(); return; }, _ => {} } }

        match key.code {
            KeyCode::Esc if self.filename_prompt == true => {
                self.filename_input.clear();
                self.pending_quit_after_save = false;
                self.filename_prompt = false;
                self.show_status("canceled file write".to_string());
            }

            KeyCode::Char('e') if
                    key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                self.toggle_explorer();
            }

            KeyCode::Char('w') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) && key.modifiers.contains(crossterm::event::KeyModifiers::ALT) => {
                if self.is_dirty() {
                    self.show_status("file is unsaved".to_string());
                } else {
                    self.exit = true;
                }
            }

            KeyCode::Char('w') if self.filename_prompt != true
                && key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                self.close_current_tab();
            }

            KeyCode::Char('s') if self.filename_prompt != true
                && key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                self.pending_quit_after_save = false;

                if !self.current_file.is_empty() {
                    if let Err(_) = self.document.save(&self.current_file) {
                        self.show_status("file does not exist".to_string());
                    } else {
                        self.last_saved = self.document.lines.clone();
                        self.show_status(format!("Saved to file: {}", &self.current_file));
                    }
                } else {
                    self.filename_prompt = true;
                    self.filename_input.clear();
                }
            }

            KeyCode::Char('z') if self.filename_prompt != true
                && key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL)
                && key.modifiers.contains(crossterm::event::KeyModifiers::SHIFT) => {
                self.redo();
            }

            KeyCode::Char('z') if self.filename_prompt != true
                && key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                self.undo();
            }

            KeyCode::Char(c) if self.filename_prompt == true => {
                self.filename_input.push(c);
            }
            KeyCode::Backspace if self.filename_prompt == true => {
                self.filename_input.pop();
            }
            KeyCode::Enter if self.filename_prompt == true => {
                if !self.filename_input.is_empty() {
                    self.current_file = self.filename_input.clone();
                    match self.document.save(&self.current_file) {
                        Ok(()) => {
                            self.last_saved = self.document.lines.clone();
                            if self.pending_quit_after_save {
                                self.exit = true;
                            }
                        }
                        Err(_) => { self.show_status("failed to save".to_string()); }
                    }
                }
                self.filename_input.clear();
                self.pending_quit_after_save = false;
                self.filename_prompt = false;
            }

            KeyCode::Char(c) => {
                let at_word_start = c.is_whitespace()
                    || self.cursor.x == 0
                    || self
                        .document
                        .lines[self.cursor.y]
                        .chars()
                        .nth(self.cursor.x - 1)
                        .map(|prev| prev.is_whitespace())
                        .unwrap_or(true);

                if self.last_action != ActionKind::Insert
                    || (at_word_start && !c.is_whitespace())
                {
                    self.push_undo();
                }

                self.document.insert_char(self.cursor.x, self.cursor.y, c);
                self.cursor.x += 1;
                if c == '(' { self.document.insert_char(self.cursor.x, self.cursor.y, ')'); }

                self.document_changed(!c.is_whitespace());

                self.last_action = ActionKind::Insert;
            }
            KeyCode::Enter => {
                self.push_undo();
                self.document.split_line(self.cursor.x, self.cursor.y);
                self.cursor.y += 1;
                self.cursor.x = 0;
                self.document_changed(false);
                self.last_action = ActionKind::Newline;
            }
            KeyCode::Backspace => {
                if self.last_action != ActionKind::Delete {
                    self.push_undo();
                }
                let (x, y) = self.document.backspace(self.cursor.x, self.cursor.y);
                self.cursor.x = x;
                self.cursor.y = y;
                self.document_changed(false);
                self.last_action = ActionKind::Delete;
            }
            KeyCode::Up => {
                self.hover = None; self.hover_pending = None; self.hover_position = None;
                self.cursor.y = self.cursor.y.saturating_sub(1);
                self.cursor.x = self.cursor.x.min(self.document.lines[self.cursor.y].len());
                self.last_action = ActionKind::None;
            }
            KeyCode::Down => {
                self.hover = None; self.hover_pending = None; self.hover_position = None;
                if self.cursor.y + 1 < self.document.lines.len() {
                    self.cursor.y += 1;
                    self.cursor.x = self.cursor.x.min(self.document.lines[self.cursor.y].len());
                }
                self.last_action = ActionKind::None;
            }
            KeyCode::Left => {
                self.hover = None; self.hover_pending = None; self.hover_position = None;
                self.cursor.x = self.cursor.x.saturating_sub(1);
                self.last_action = ActionKind::None;
            }
            KeyCode::Right => {
                self.hover = None; self.hover_pending = None; self.hover_position = None;
                let line_length = self.document.lines[self.cursor.y].len();
                if self.cursor.x < line_length {
                    self.cursor.x += 1;
                }
                self.last_action = ActionKind::None;
            }

            _ => {}
        }

        self.update_scroll(self.viewport_height.get());
    }

    fn accept_completion(&mut self) { let Some(item) = self.completions.get(self.completion_selected) else { return; }; let mut text = item.insert_text.clone().unwrap_or_else(|| item.label.clone()); let mut cursor = text.len(); if matches!(item.kind, Some(2..=4)) { if !text.contains('(') { text.push_str("()"); } cursor = text.find('(').map_or(text.len(), |index| index + 1); } let start = self.document.lines[self.cursor.y][..self.cursor.x].rfind(|c: char| !c.is_alphanumeric() && c != '_').map_or(0, |index| index + 1); self.document.lines[self.cursor.y].replace_range(start..self.cursor.x, &text); self.cursor.x = start + cursor; self.completions.clear(); self.document_changed(false); }

    pub fn handle_mouse(&mut self, mouse: MouseEvent) {
        match mouse.kind {
            MouseEventKind::Moved => self.request_hover_at(mouse.column, mouse.row),
            MouseEventKind::Down(MouseButton::Left) => {
                if Self::point_in_rect(mouse.column, mouse.row, self.tabs_area.get()) {
                    self.start_tab_drag(mouse);
                } else if self.show_explorer
                    && Self::point_in_rect(mouse.column, mouse.row, self.explorer_area.get())
                {
                    self.start_entry_drag(mouse);
                }
            }
            MouseEventKind::Drag(MouseButton::Left) => {
                if self.dragging_tab.is_some() {
                    self.update_tab_drag(mouse);
                } else if self.dragging_entry.is_some() {
                    self.update_entry_drag(mouse);
                }
            }
            MouseEventKind::Up(MouseButton::Left) => {
                if self.dragging_tab.is_some() {
                    self.finish_tab_drag(mouse);
                } else if self.dragging_entry.is_some() {
                    self.finish_entry_drag(mouse);
                }
            }

            MouseEventKind::ScrollUp => {
                self.cursor.y = self.cursor.y.saturating_sub(3);
                self.update_scroll(self.viewport_height.get());
            }
            MouseEventKind::ScrollDown => {
                let max_y = self.document.lines.len().saturating_sub(1);
                self.cursor.y = (self.cursor.y + 3).min(max_y);
                self.update_scroll(self.viewport_height.get());
            }
            _ => {}
        }
    }
}