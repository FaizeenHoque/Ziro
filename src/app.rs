use std::io;
use std::cell::Cell;
use std::time::Duration;

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
    cursor::Cursor, document::{Document}, syntax::Highlighter, ui,
};


#[derive(Debug)]
pub struct App {
    pub document: Document,
    pub cursor: Cursor,
    pub highlighter: Highlighter,
    
    pub undo_stack: Vec<UndoState>,
    pub redo_stack: Vec<UndoState>,
    pub last_action: ActionKind,
    pub last_saved: Vec<String>,
    
    pub scroll_y: usize,
    pub viewport_height: Cell<usize>,
    pub number_col_width: u16,
    
    pub current_file: String,
    pub filename_input: String,
    
    pub status_text: String,
    pub status: bool,
    pub filename_prompt: bool,
    pub show_explorer: bool,
    
    pub pending_quit_after_save: bool,
    pub exit: bool,
}

#[derive(Debug, Clone)]
pub struct UndoState {
    pub lines: Vec<String>,
    pub cursor_x: usize,
    pub cursor_y: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActionKind {
    None,
    Insert,
    Delete,
    Newline,
}

impl std::fmt::Debug for Highlighter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Highlighter").finish()
    }
}

impl Default for App {
    fn default() -> Self {
        let document = Document::default();
        let last_saved = document.lines.clone();
        Self {
            document,
            current_file: String::new(),
            highlighter: Highlighter::new(),
            cursor: Cursor::default(),
            
            viewport_height: Cell::new(20),
            number_col_width: 7,
            scroll_y: 0,
            
            filename_input: String::new(),
            status_text: String::new(),
            
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            last_action: ActionKind::None,
            last_saved,
            
            filename_prompt: false,
            show_explorer: false,
            
            pending_quit_after_save: false,
            status: false,
            exit: false,
        }
    }
}

pub const EXPLORER_WIDTH: u16 = 28;

impl App {
    pub fn new(file: Option<String>) -> io::Result<Self> {
        let mut app = Self::default();
        if let Some(filename) = file {
            app.current_file = filename.clone();
            app.document = Document::from_file(&filename)?;
            app.last_saved = app.document.lines.clone();
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

    pub fn editor_x_offset(&self) -> u16 {
        if self.show_explorer { EXPLORER_WIDTH } else { 0 }
    }
    
    fn draw(&mut self, frame: &mut Frame) {
        ui::draw(frame, self);
        let area = frame.area();
        
        if self.filename_prompt == true  {
            let popup_x = area.width.saturating_sub(40) / 2;
            let popup_y = area.height.saturating_sub(5) / 2;
            frame.set_cursor_position((
                popup_x + 1 + self.filename_input.len() as u16,
                popup_y + 2,
            ));
        } else {
            let x = self.editor_x_offset() + self.number_col_width + self.cursor.x as u16;
            let y = 1 + (self.cursor.y - self.scroll_y) as u16;

            frame.set_cursor_position((
                x.min(area.width.saturating_sub(1)),
                y.min(area.height.saturating_sub(2)),
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
        
        while event::poll(Duration::from_millis(0))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    self.handle_key(key);
                }
            }
        }
        
        Ok(())
    }
    
    pub fn is_dirty(&self) -> bool {
        self.document.lines != self.last_saved
    }

    fn handle_key(&mut self, key: crossterm::event::KeyEvent) {
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

            KeyCode::Char('w') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL)
                && key.modifiers.contains(crossterm::event::KeyModifiers::ALT) => {
                self.exit = true;
            }

            KeyCode::Char('w') if self.filename_prompt != true
                && key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                if self.is_dirty() {
                    self.show_status("file is unsaved".to_string());
                } else {
                    self.exit = true;
                }
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


                if self.last_action != ActionKind::Insert || (at_word_start && !c.is_whitespace()) {
                    self.push_undo();
                }

                self.document.insert_char(self.cursor.x, self.cursor.y, c);
                self.cursor.x += 1;
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

    pub fn update_scroll(&mut self, viewport_height: usize) {
        if self.cursor.y < self.scroll_y {
            self.scroll_y = self.cursor.y;
        }

        if self.cursor.y >= self.scroll_y + viewport_height {
            self.scroll_y = self.cursor.y - viewport_height + 1;
        }
    }

    pub fn show_status(&mut self, text: String) {
        self.status = true;
        self.status_text = text;
    }

    pub fn reset_status(&mut self) {
        self.status = false;
        self.status_text = String::new();
    }

    fn snapshot(&self) -> UndoState {
        UndoState { lines: self.document.lines.clone(), cursor_x: self.cursor.x, cursor_y: self.cursor.y }
    }

    pub fn push_undo(&mut self) {
        self.undo_stack.push(self.snapshot());
        self.redo_stack.clear();
    }

    pub fn undo(&mut self) {
        match self.undo_stack.pop() {
            Some(state) => {
                self.redo_stack.push(self.snapshot());
                self.document.lines = state.lines;
                self.cursor.x = state.cursor_x;
                self.cursor.y = state.cursor_y;
                self.last_action = ActionKind::None;
            }
            None => self.show_status("nothing to undo".to_string()),
        }
    }

    pub fn redo(&mut self) {
        match self.redo_stack.pop() {
            Some(state) => {
                self.undo_stack.push(self.snapshot());
                self.document.lines = state.lines;
                self.cursor.x = state.cursor_x;
                self.cursor.y = state.cursor_y;
                self.last_action = ActionKind::None;
            }
            None => self.show_status("nothing to redo".to_string()),
        }
    }

    pub fn toggle_explorer(&mut self) {
        if self.show_explorer == false {
            self.show_explorer = true;
            self.show_status("Opened Explorer".to_string());
        } else {
            self.show_explorer = false;
            self.show_status("Closed Explorer".to_string());
        }
    }

}
