use std::path::{Path, PathBuf};

use crossterm::event::MouseEvent;

use crate::{app::{App}, editor::{Cursor, Document}};


#[derive(Debug, Clone)]
pub struct TabItem {
    pub name: String,
    pub path: PathBuf,
    pub cursor: Cursor,
    pub scroll_y: usize,
}

impl App {
    pub fn tab_index_at(&self, column: u16, row: u16) -> Option<usize> {
        let area = self.tabs_area.get();
        if row != area.y { return None; }

        let mut x = area.x;
        for (i, tab) in self.tabs_list.iter().enumerate() {
            let tab_width = tab.name.len() as u16 + 4;
            if column >= x && column < x + tab_width {
                return Some(i);
            }
            x+=tab_width;
        }
        None
    }

    pub fn start_tab_drag(&mut self, mouse: MouseEvent) {
        if let Some(index) = self.tab_index_at(mouse.column, mouse.row) {
            self.dragging_tab = Some(index);
            self.tab_drag_target = Some(index);
        }
    }

    pub fn update_tab_drag(&mut self, mouse: MouseEvent) {
        if let Some(index) = self.tab_index_at(mouse.column, mouse.row) {
            self.tab_drag_target = Some(index);
        } else if mouse.row == self.tabs_area.get().y {
            self.tab_drag_target = Some(self.tabs_list.len().saturating_sub(1));
        }
    }

    pub fn finish_tab_drag(&mut self, mouse: MouseEvent) {
        let start_index = self.dragging_tab.take().unwrap();
        let target_index = self.tab_drag_target.take();

        match target_index {
            Some(t) if t != start_index && t < self.tabs_list.len() => {
                let tab = self.tabs_list.remove(start_index);
                self.tabs_list.insert(t, tab);
            }
            _=> self.click_tab(start_index, mouse.column),
        }
    }

    pub fn click_tab(&mut self, index: usize, column: u16) {
        let tab = match self.tabs_list.get(index).cloned() {
            Some(t) => t,
            None => return
        };

        let area = self.tabs_area.get();
        let x_start = area.x
            + self.tabs_list[..index]
                .iter()
                .map(|t| t.name.len() as u16 + 4)
                .sum::<u16>();
        let tab_width = tab.name.len() as u16 + 4;
        let close_glyph_x = x_start + tab_width - 2;

        if column == close_glyph_x {
            self.close_tab(&tab.path);
        } else {
            self.switch_to_file(tab);
        }
    }

    pub fn save_current_tab_state(&mut self) {
        if let Some(tab) = self
            .tabs_list
            .iter_mut()
            .find(|t| t.path.to_string_lossy() == self.current_file)
        {
            tab.cursor.x = self.cursor.x;
            tab.cursor.y = self.cursor.y;
            tab.scroll_y = self.scroll_y;
        }
    }

    pub fn push_file_to_tabs(&mut self, path: &Path) {
        if let Some(existing) = self.tabs_list.iter().find(|t| t.path == path) {
            let tab_snapshot = existing.clone();
            self.switch_to_file(tab_snapshot);
            return;
        }

        let name = path.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

        let tab = TabItem { 
            name, path: path.to_path_buf(), 
            cursor: Cursor { x: self.cursor.x, y: self.cursor.y },
            scroll_y: self.scroll_y
        };

        let tab_snapshot = tab.clone();
        self.tabs_list.push(tab);
        self.switch_to_file(tab_snapshot);
    }

    pub fn close_current_tab(&mut self) {
        let idx = match self.tabs_list.iter().position(|t| t.path.to_string_lossy() == self.current_file) {
            Some(i) => i,
            None => return, // current_file isn't even an open tab, nothing to do
        };

        self.tabs_list.remove(idx);

        if self.tabs_list.is_empty() {
            self.document = Document::default();
            self.current_file = String::new();
            self.last_saved = self.document.lines.clone();
            self.cursor.x = 0;
            self.cursor.y = 0;
            self.scroll_y = 0;
            return;
        }

        // if we removed the last tab in the list, step back one; otherwise
        // the tab that shifted into idx is the "next" one
        let next_idx = idx.min(self.tabs_list.len() - 1);
        self.switch_to_file(self.tabs_list[next_idx].clone());
    }

    pub fn close_tab(&mut self, path: &Path) {
        let idx = match self.tabs_list.iter().position(|t| &t.path == path) {
            Some(i) => i,
            None => return,
        };

        let was_current = self.current_file == path.to_string_lossy();
        self.tabs_list.remove(idx);

        if !was_current {
            return; // closing a background tab shouldn't touch the active document
        }

        if self.tabs_list.is_empty() {
            self.document = Document::default();
            self.current_file = String::new();
            self.last_saved = self.document.lines.clone();
            self.cursor.x = 0;
            self.cursor.y = 0;
            self.scroll_y = 0;
            return;
        }

        let next_idx = idx.min(self.tabs_list.len() - 1);
        self.switch_to_file(self.tabs_list[next_idx].clone());
    }
}