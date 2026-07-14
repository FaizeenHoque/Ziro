use crossterm::event::{KeyCode};
use crate::app::{ActionKind, App};

impl App {

    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) {
        if self.status {
            self.reset_status();
        }
        if !self.completions.is_empty() { match key.code { KeyCode::Esc => { self.completions.clear(); return; }, KeyCode::Up => { self.completion_selected = self.completion_selected.saturating_sub(1); return; }, KeyCode::Down => { self.completion_selected = (self.completion_selected + 1).min(self.completions.len() - 1); return; }, KeyCode::Enter | KeyCode::Tab => { self.accept_completion(); return; }, _ => {} } }

        match key.code {
                        KeyCode::Backspace | KeyCode::Char('h') if self.filename_prompt != true 
                && key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                if self.last_action != ActionKind::Delete {
                    self.push_undo();
                }

                while self.cursor.x > 0 {
                    let ch = self.document.lines[self.cursor.y]
                        .chars()
                        .nth(self.cursor.x - 1)
                        .unwrap();

                    if !ch.is_whitespace() {
                        break;
                    }

                    let (x, y) = self.document.backspace(self.cursor.x, self.cursor.y);
                    self.cursor.x = x;
                    self.cursor.y = y;
                }

                while self.cursor.x > 0 {
                    let ch = self.document.lines[self.cursor.y]
                        .chars()
                        .nth(self.cursor.x - 1)
                        .unwrap();

                    if ch.is_whitespace() {
                        break;
                    }

                    let (x, y) = self.document.backspace(self.cursor.x, self.cursor.y);
                    self.cursor.x = x;
                    self.cursor.y = y;
                }

                self.document_changed(false);
                self.last_action = ActionKind::Delete;
            }
            KeyCode::Char(c) if self.filename_prompt == true 
                && !key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) 
                && !key.modifiers.contains(crossterm::event::KeyModifiers::ALT) => {
                self.filename_input.push(c);
            }
            KeyCode::Char(c) if !key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) 
                             && !key.modifiers.contains(crossterm::event::KeyModifiers::ALT) => {
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
            KeyCode::Tab => {
                self.document.insert_char(self.cursor.x, self.cursor.y, ' ');
                self.document.insert_char(self.cursor.x, self.cursor.y, ' ');
                self.document.insert_char(self.cursor.x, self.cursor.y, ' ');
                self.document.insert_char(self.cursor.x, self.cursor.y, ' ');
                self.cursor.x += 4;
            }
            KeyCode::Enter => {
                self.push_undo();
                self.document.split_line(self.cursor.x, self.cursor.y);
                self.cursor.y += 1;
                self.cursor.x = 0;
                self.document_changed(false);
                self.last_action = ActionKind::Newline;
            }
            KeyCode::Backspace if self.filename_prompt != true 
                && key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                if self.last_action != ActionKind::Delete {
                    self.push_undo();
                }

                while self.cursor.x > 0 {
                    let ch = self.document.lines[self.cursor.y]
                        .chars()
                        .nth(self.cursor.x - 1)
                        .unwrap();

                    if !ch.is_whitespace() {
                        break;
                    }

                    let (x, y) = self.document.backspace(self.cursor.x, self.cursor.y);
                    self.cursor.x = x;
                    self.cursor.y = y;
                }

                while self.cursor.x > 0 {
                    let ch = self.document.lines[self.cursor.y]
                        .chars()
                        .nth(self.cursor.x - 1)
                        .unwrap();

                    if ch.is_whitespace() {
                        break;
                    }

                    let (x, y) = self.document.backspace(self.cursor.x, self.cursor.y);
                    self.cursor.x = x;
                    self.cursor.y = y;
                }

                self.document_changed(false);
                self.last_action = ActionKind::Delete;
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
}