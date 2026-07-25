use crate::app::{ActionKind, App};

#[derive(Debug, Clone)]
pub struct UndoState {
    pub lines: Vec<String>,
    pub cursor_x: usize,
    pub cursor_y: usize,
}

impl App {
    fn snapshot(&self) -> UndoState {
        UndoState {
            lines: self.document.lines.clone(),
            cursor_x: self.cursor.x,
            cursor_y: self.cursor.y,
        }
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
}
