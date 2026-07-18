use crate::app::{ActionKind, App};
use crossterm::event::{KeyCode, KeyModifiers};

impl App {
    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) {
        if self.status {
            self.reset_status();
        }

        if !self.completions.is_empty() {
            match key.code {
                KeyCode::Esc => {
                    self.completions.clear();
                    return;
                }
                KeyCode::Up => {
                    self.completion_selected = self.completion_selected.saturating_sub(1);
                    return;
                }
                KeyCode::Down => {
                    self.completion_selected =
                        (self.completion_selected + 1).min(self.completions.len() - 1);
                    return;
                }
                KeyCode::Enter | KeyCode::Tab => {
                    self.accept_completion();
                    return;
                }
                _ => {}
            }
        }

        if self.filename_prompt {
            self.handle_filename_prompt_key(key);
            self.update_scroll(self.viewport_height.get());
            return;
        }

        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                todo!("Implement Copy");
            }

            KeyCode::Char('v') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.paste();
            }

            KeyCode::Char('x') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                todo!("Implement Cut");
            }

            KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.toggle_explorer();
            }

            KeyCode::Char('w')
                if key.modifiers.contains(KeyModifiers::CONTROL)
                    && key.modifiers.contains(KeyModifiers::ALT) =>
            {
                if self.is_dirty() {
                    self.show_status("file is unsaved".to_string());
                } else {
                    self.exit = true;
                }
            }

            KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.close_current_tab();
            }

            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
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

            KeyCode::Char('z')
                if key.modifiers.contains(KeyModifiers::CONTROL)
                    && key.modifiers.contains(KeyModifiers::SHIFT) =>
            {
                self.redo();
            }

            KeyCode::Char('z') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.undo();
            }

            KeyCode::Backspace | KeyCode::Char('h')
                if key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                self.delete_word_before_cursor();
            }

            KeyCode::Char(c) => {
                let at_word_start = c.is_whitespace()
                    || self.cursor.x == 0
                    || self.document.lines[self.cursor.y]
                        .chars()
                        .nth(self.cursor.x - 1)
                        .map(|prev| prev.is_whitespace())
                        .unwrap_or(true);

                if self.last_action != ActionKind::Insert || (at_word_start && !c.is_whitespace()) {
                    self.push_undo();
                }

                self.document.insert_char(self.cursor.x, self.cursor.y, c);
                self.cursor.x += 1;

                let closing = match c {
                    '(' => Some(')'),
                    '{' => Some('}'),
                    '[' => Some(']'),
                    '"' => Some('"'),
                    '`' => Some('`'),
                    '\'' => Some('\''),
                    _ => None,
                };
                if let Some(close_ch) = closing {
                    self.document
                        .insert_char(self.cursor.x, self.cursor.y, close_ch);
                }

                self.document_changed(!c.is_whitespace());
                self.last_action = ActionKind::Insert;
            }

            KeyCode::Tab => {
                for _ in 0..4 {
                    self.document.insert_char(self.cursor.x, self.cursor.y, ' ');
                }
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
                self.clear_hover();
                self.cursor.y = self.cursor.y.saturating_sub(1);
                self.cursor.x = self.cursor.x.min(self.document.lines[self.cursor.y].len());
                self.last_action = ActionKind::None;
            }
            KeyCode::Down => {
                self.clear_hover();
                if self.cursor.y + 1 < self.document.lines.len() {
                    self.cursor.y += 1;
                    self.cursor.x = self.cursor.x.min(self.document.lines[self.cursor.y].len());
                }
                self.last_action = ActionKind::None;
            }
            KeyCode::Left => {
                self.clear_hover();
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    if self.cursor.x == 0 {
                        if self.cursor.y > 0 {
                            self.cursor.y -= 1;
                            self.cursor.x = self.document.lines[self.cursor.y].len();
                        }
                    } else {
                        self.cursor.x =
                            Self::word_left(&self.document.lines[self.cursor.y], self.cursor.x);
                    }
                } else if self.cursor.x == 0 {
                    if self.cursor.y > 0 {
                        self.cursor.y -= 1;
                        self.cursor.x = self.document.lines[self.cursor.y].len();
                    }
                } else {
                    self.cursor.x -= 1;
                }
                self.last_action = ActionKind::None;
            }
            KeyCode::Right => {
                self.clear_hover();
                let line_length = self.document.lines[self.cursor.y].len();
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    if self.cursor.x >= line_length {
                        if self.cursor.y + 1 < self.document.lines.len() {
                            self.cursor.y += 1;
                            self.cursor.x = 0;
                        }
                    } else {
                        self.cursor.x =
                            Self::word_right(&self.document.lines[self.cursor.y], self.cursor.x);
                    }
                } else if self.cursor.x < line_length {
                    self.cursor.x += 1;
                } else if self.cursor.y + 1 < self.document.lines.len() {
                    self.cursor.y += 1;
                    self.cursor.x = 0;
                }
                self.last_action = ActionKind::None;
            }

            _ => {}
        }

        self.update_scroll(self.viewport_height.get());
    }

    fn handle_filename_prompt_key(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Char(c)
                if !key.modifiers.contains(KeyModifiers::CONTROL)
                    && !key.modifiers.contains(KeyModifiers::ALT) =>
            {
                self.filename_input.push(c);
            }

            KeyCode::Backspace => {
                self.filename_input.pop();
            }

            KeyCode::Esc => {
                self.filename_input.clear();
                self.pending_quit_after_save = false;
                self.filename_prompt = false;
                self.show_status("canceled file write".to_string());
            }

            KeyCode::Enter => {
                if !self.filename_input.is_empty() {
                    self.current_file = self.filename_input.clone();
                    match self.document.save(&self.current_file) {
                        Ok(()) => {
                            self.last_saved = self.document.lines.clone();
                            if self.pending_quit_after_save {
                                self.exit = true;
                            }
                        }
                        Err(_) => {
                            self.show_status("failed to save".to_string());
                        }
                    }
                }
                self.filename_input.clear();
                self.pending_quit_after_save = false;
                self.filename_prompt = false;
            }

            _ => {}
        }
    }

    fn delete_word_before_cursor(&mut self) {
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
}

impl App {
    fn word_left(line: &str, idx: usize) -> usize {
        let head = &line[..idx];
        let mut offset = 0usize;
        let mut chars = head.chars().rev().peekable();

        while let Some(&c) = chars.peek() {
            if c.is_whitespace() {
                offset += c.len_utf8();
                chars.next();
            } else {
                break;
            }
        }

        if let Some(&c0) = chars.peek() {
            let is_word = |c: char| c.is_alphanumeric() || c == '_';
            let starting_is_word = is_word(c0);
            while let Some(&c) = chars.peek() {
                if !c.is_whitespace() && is_word(c) == starting_is_word {
                    offset += c.len_utf8();
                    chars.next();
                } else {
                    break;
                }
            }
        }

        idx - offset
    }

    fn word_right(line: &str, idx: usize) -> usize {
        let rest = &line[idx..];
        let mut offset = 0usize;
        let mut chars = rest.chars().peekable();

        while let Some(&c) = chars.peek() {
            if c.is_whitespace() {
                offset += c.len_utf8();
                chars.next();
            } else {
                break;
            }
        }

        if let Some(&c0) = chars.peek() {
            let is_word = |c: char| c.is_alphanumeric() || c == '_';
            let starting_is_word = is_word(c0);
            while let Some(&c) = chars.peek() {
                if !c.is_whitespace() && is_word(c) == starting_is_word {
                    offset += c.len_utf8();
                    chars.next();
                } else {
                    break;
                }
            }
        }

        idx + offset
    }

    fn clear_hover(&mut self) {
        self.hover = None;
        self.hover_pending = None;
        self.hover_position = None;
    }

    fn paste(&mut self) {
        let mut clipboard = match arboard::Clipboard::new() {
            Ok(c) => c,
            Err(_) => {
                self.show_status("clipboard unavailable".to_string());
                return;
            }
        };

        let text = match clipboard.get_text() {
            Ok(t) => t,
            Err(e) => {
                self.show_status(format!("clipboard error: {e}"));
                return;
            }
        };

        if text.is_empty() {
            return;
        }

        self.push_undo();

        let (new_x, new_y) = self
            .document
            .insert_str(self.cursor.x, self.cursor.y, &text);
        self.cursor.x = new_x;
        self.cursor.y = new_y;

        self.document_changed(true);
        self.last_action = ActionKind::Insert;
    }
}
