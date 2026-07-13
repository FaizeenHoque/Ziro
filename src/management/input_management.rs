use std::io;
use std::time::Duration;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, MouseButton, MouseEvent, MouseEventKind};

use crate::app::{ActionKind, App};
use crate::lsp::LspMessage;

impl App {
    pub fn handle_events(&mut self) -> io::Result<()> {
        if self.poll_and_dispatch(Duration::from_millis(16))? {
            while self.poll_and_dispatch(Duration::from_millis(0))? {}
        }

        // Drain all pending messages from rust-analyzer.
        // NOTE: only LspMessage::Completion is handled right now; the rest
        // (Diagnostics, Hover, Definition, Initialized, Json) are not yet
        // wired to any UI state. Add arms here as those features land.
        if let Some(lsp) = self.lsp.as_mut() {
            while let Some(msg) = lsp.try_recv() {
                if let LspMessage::Completion(items) = msg {
                    crate::debug::log(format!("{} completion items", items.len()));
                    for item in items {
                        crate::debug::log(format!("  {}", item.label));
                    }
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

                let path = if self.current_file.is_empty() {
                    None
                } else {
                    Some(std::path::PathBuf::from(&self.current_file))
                };

                if let (Some(path), Some(lsp)) = (path, self.lsp.as_mut()) {
                    let text = self.document.lines.join("\n");

                    if let Err(e) = lsp.did_change(&path, &text) {
                        crate::debug::log(format!("did_change failed: {e}"));
                    }

                    if let Err(e) = lsp.completion(&path, self.cursor.y, self.cursor.x) {
                        crate::debug::log(format!("completion failed: {e}"));
                    }
                }

                self.last_action = ActionKind::Insert;
            }
            KeyCode::Enter => {
                self.push_undo();
                self.document.split_line(self.cursor.x, self.cursor.y);
                self.cursor.y += 1;
                self.cursor.x = 0;
                self.last_action = ActionKind::Newline;
            }
            KeyCode::Backspace => {
                if self.last_action != ActionKind::Delete {
                    self.push_undo();
                }
                let (x, y) = self.document.backspace(self.cursor.x, self.cursor.y);
                self.cursor.x = x;
                self.cursor.y = y;
                self.last_action = ActionKind::Delete;
            }
            KeyCode::Up => {
                self.cursor.y = self.cursor.y.saturating_sub(1);
                self.cursor.x = self.cursor.x.min(self.document.lines[self.cursor.y].len());
                self.last_action = ActionKind::None;
            }
            KeyCode::Down => {
                if self.cursor.y + 1 < self.document.lines.len() {
                    self.cursor.y += 1;
                    self.cursor.x = self.cursor.x.min(self.document.lines[self.cursor.y].len());
                }
                self.last_action = ActionKind::None;
            }
            KeyCode::Left => {
                self.cursor.x = self.cursor.x.saturating_sub(1);
                self.last_action = ActionKind::None;
            }
            KeyCode::Right => {
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

    pub fn handle_mouse(&mut self, mouse: MouseEvent) {
        match mouse.kind {
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
