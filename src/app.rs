use std::io;
use std::cell::Cell;

use crossterm::event::{
    self,
    Event,
    KeyCode,
    KeyEventKind,
};

use ratatui::{
    DefaultTerminal,
    Frame,
};

use crate::{
    cursor::Cursor,
    document::Document,
    mode::Mode,
    ui,
};

#[derive(Debug)]
pub struct App {
    pub document: Document,
    pub cursor: Cursor,

    pub scroll_y: usize,
    pub viewport_height: Cell<usize>,

    exit: bool,
    pub mode: Mode,
    pub status_text: String,
    pub command_input: String,
    pub current_file: String,
    pub dirty: bool, 

    pub warning: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            document: Document::default(),
            cursor: Cursor::default(),
            scroll_y: 0,
            viewport_height: Cell::new(20),
            exit: false,
            mode: Mode::default(),
            status_text: String::new(),
            command_input: String::new(),
            current_file: String::new(),
            dirty: false,
            warning: false,
        }
    }
}

impl App {
    pub fn new(file: Option<String>) -> io::Result<Self> {
        let mut app = Self::default();
        if let Some(filename) = file {
            app.current_file = filename.clone();
            app.document = Document::from_file(&filename)?;
        }
        Ok(app)
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal,) -> io::Result<()> {

        while !self.exit {

            terminal.draw(|frame| {
                self.draw(frame);
            })?;

            self.handle_events()?;
        }

        Ok(())
    }

    pub fn show_warning(&mut self, text: String) {
        self.warning = true;
        self.status_text = text;
    }

    pub fn reset_warning(&mut self) {
        self.warning = false;
        self.status_text = String::new();
    }

    fn draw(
        &mut self,
        frame: &mut Frame,
    ) {
        ui::draw(frame, self);

        let area = frame.area();

        if self.mode == Mode::Command {
            frame.set_cursor_position((
                self.command_input.len() as u16,
                area.height - 1,
            ));
        } else {
            frame.set_cursor_position((
                (self.cursor.x + 5) as u16,
                (self.cursor.y - self.scroll_y) as u16,
            ));
        }
            
    }
    

    fn handle_events(
        &mut self
    ) -> io::Result<()> {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                self.handle_key(key);
            }
        }
        Ok(())
    }

    fn handle_key(&mut self, key: crossterm::event::KeyEvent,) {
        match key.code {

            // Document Typing
            KeyCode::Char(c) if self.mode == Mode::Insert => {
                self.document.insert_char(
                    self.cursor.x,
                    self.cursor.y,
                    c,
                );

                self.cursor.x += 1;
                self.dirty = true;
            }
            KeyCode::Enter if self.mode == Mode::Insert => {
                self.document.split_line(
                    self.cursor.x,
                    self.cursor.y,
                );

                self.cursor.y += 1;
                self.cursor.x = 0;
                self.dirty = true;
            }
            KeyCode::Backspace if self.mode == Mode::Insert => {
                let (x, y) = self.document.backspace(
                    self.cursor.x,
                    self.cursor.y,
                );

                self.cursor.x = x;
                self.cursor.y = y;
                self.dirty = true;
            }

            // Document Movement Keybinds
            KeyCode::Up if self.mode == Mode::Insert || self.mode == Mode::Normal => {
                self.cursor.y = self.cursor.y.saturating_sub(1);

                self.cursor.x = self.cursor.x
                    .min(self.document.lines[self.cursor.y].len());
            }
            KeyCode::Down if self.mode == Mode::Insert || self.mode == Mode::Normal => {
                if self.cursor.y + 1 < self.document.lines.len() {
                    self.cursor.y += 1;

                    self.cursor.x = self.cursor.x
                        .min(self.document.lines[self.cursor.y].len());
                }
            }
            KeyCode::Left if self.mode == Mode::Insert || self.mode == Mode::Normal => {
                self.cursor.x = self.cursor.x.saturating_sub(1);
            }
            KeyCode::Right if self.mode == Mode::Insert || self.mode == Mode::Normal => {
                let line_length = self.document.lines[self.cursor.y].len();

                if self.cursor.x < line_length {
                    self.cursor.x += 1;
                }
            }

            KeyCode::Char('k') if self.mode == Mode::Normal => {
                self.cursor.y = self.cursor.y.saturating_sub(1);

                self.cursor.x = self.cursor.x
                    .min(self.document.lines[self.cursor.y].len());
            }
            KeyCode::Char('j') if self.mode == Mode::Normal => {
                if self.cursor.y + 1 < self.document.lines.len() {
                    self.cursor.y += 1;

                    self.cursor.x = self.cursor.x
                        .min(self.document.lines[self.cursor.y].len());
                }
            }
            KeyCode::Char('h') if self.mode == Mode::Normal => {
                self.cursor.x = self.cursor.x.saturating_sub(1);
            }
            KeyCode::Char('l') if self.mode == Mode::Normal => {
                let line_length = self.document.lines[self.cursor.y].len();

                if self.cursor.x < line_length {
                    self.cursor.x += 1;
                }
            }

            // Command Mode Keybinds
            KeyCode::Char(c) if self.mode == Mode::Command => {
                self.command_input.push(c);
            }
            KeyCode::Backspace if self.mode == Mode::Command => {
                self.command_input.pop();
            }
            KeyCode::Enter if self.mode == Mode::Command => {
                match self.command_input.as_str() {
                    ":q" => {
                        if self.dirty {
                            self.show_warning("file is unsaved".to_string());
                        } else {
                            self.exit = true;
                        }
                    }
                    ":!q" => {
                        self.exit = true;
                    }
                    ":w" => {
                        if !self.current_file.is_empty() {
                            if let Err(_) = self.document.save(&self.current_file) {
                                self.show_warning("file does not exist".to_string());
                            } else {
                                self.dirty = false;
                            }
                        } else {
                            // TODO: Prompt for filename
                        }
                    }
                    ":wq" | ":x" => {
                        if !self.current_file.is_empty() {
                            match self.document.save(&self.current_file) {
                                Ok(()) => {
                                    self.dirty = false;
                                    self.exit = true;
                                }
                                Err(_) => {
                                    self.show_warning("failed to save file".to_string());
                                }
                            }
                        } else {
                            self.show_warning("no file specified".to_string());
                        }
                    }
                    _ => {}
                }

                self.command_input.clear();
                self.mode = Mode::Normal;
            }

            // Mode Switching
            KeyCode::Esc => {
                self.mode = match self.mode {
                    Mode::Normal => Mode::Normal,
                    Mode::Insert => Mode::Normal,
                    Mode::Command => Mode::Normal,
                };

                self.command_input.clear();
            }
            
            KeyCode::Char('i') if self.mode == Mode::Normal => {
                self.mode = Mode::Insert;
            }

            KeyCode::Char(':') if self.mode == Mode::Normal => {
                self.mode = Mode::Command;
                self.command_input = ":".to_string();
            }

            _ => {}
        }

        self.update_scroll(self.viewport_height.get());
    }

    pub fn update_scroll(&mut self, viewport_height: usize) {
        if self.cursor.y < self.scroll_y {
            self.scroll_y = self.cursor.y;
        }

        if self.cursor.y >= self.scroll_y + viewport_height {
            self.scroll_y = self.cursor.y - viewport_height + 1;
        }
    }
}