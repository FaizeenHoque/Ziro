use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

use crate::app::App;

impl App {
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