use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

use crate::app::App;

impl App {
    pub fn handle_mouse(&mut self, mouse: MouseEvent) {
        match mouse.kind {
            MouseEventKind::Moved => self.request_hover_at(mouse.column, mouse.row),
            MouseEventKind::Down(MouseButton::Left) => {
                self.hover = None;
                if Self::point_in_rect(mouse.column, mouse.row, self.tabs_area.get()) {
                    self.start_tab_drag(mouse);
                } else if self.show_explorer
                    && Self::point_in_rect(mouse.column, mouse.row, self.explorer_area.get())
                {
                    self.start_entry_drag(mouse);
                } else if Self::point_in_rect(mouse.column, mouse.row, self.editor_area.get()) {
                    self.move_cursor_to_mouse(mouse.column, mouse.row);
                }
            }
            MouseEventKind::Drag(MouseButton::Left) => {
                self.hover = None;
                if self.dragging_tab.is_some() {
                    self.update_tab_drag(mouse);
                } else if self.dragging_entry.is_some() {
                    self.update_entry_drag(mouse);
                }
            }
            MouseEventKind::Up(MouseButton::Left) => {
                self.hover = None;
                if self.dragging_tab.is_some() {
                    self.finish_tab_drag(mouse);
                } else if self.dragging_entry.is_some() {
                    self.finish_entry_drag(mouse);
                }
            }

            MouseEventKind::ScrollUp => {
                self.hover = None;
                self.scroll_y = self.scroll_y.saturating_sub(3);
                let bottom = self.scroll_y + self.viewport_height.get().saturating_sub(1);
                if self.cursor.y > bottom {
                    self.cursor.y = bottom;
                    self.clamp_cursor_x();
                }
            }
            MouseEventKind::ScrollDown => {
                self.hover = None;
                let max_scroll = self
                    .document
                    .lines
                    .len()
                    .saturating_sub(self.viewport_height.get());
                self.scroll_y = (self.scroll_y + 3).min(max_scroll);
                if self.cursor.y < self.scroll_y {
                    self.cursor.y = self.scroll_y;
                    self.clamp_cursor_x();
                }
            }
            _ => {}
        }
    }

    pub fn move_cursor_to_mouse(&mut self, column: u16, row: u16) {
        let area = self.editor_area.get();
        let text_start_x = area.x + self.number_col_width;

        if column < text_start_x || self.document.lines.is_empty() {
            return;
        }

        let line_index = self.scroll_y + (row - area.y) as usize;
        let line_index = line_index.min(self.document.lines.len() - 1);

        let Some(text) = self.document.lines.get(line_index) else {
            return;
        };

        let col_offset = (column - text_start_x) as usize;
        let byte_index = text
            .char_indices()
            .nth(col_offset)
            .map(|(i, _)| i)
            .unwrap_or(text.len());

        self.cursor.y = line_index;
        self.cursor.x = byte_index;
    }
}
