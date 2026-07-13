use std::path::{Path, PathBuf};

use crossterm::event::MouseEvent;

use crate::app::{App};

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub depth: usize,
    pub expanded: bool,
}

impl App {

    pub fn start_entry_drag(&mut self, mouse: MouseEvent) {
        let area = self.explorer_area.get();
        if mouse.row < area.y { return; }

        let row = (mouse.row - area.y) as usize;
        if row < self.explorer_entries.len() {
            self.explorer_selected = row;
            self.dragging_entry = Some(row);
            self.entry_drag_target = Some(row);
        }
    }

    pub fn update_entry_drag(&mut self, mouse: MouseEvent) {
        let area = self.explorer_area.get();
        if mouse.row < area.y { return; }

        let row = (mouse.row - area.y) as usize;
        if row < self.explorer_entries.len() {
            self.entry_drag_target = Some(row);
        }
    }

    pub fn finish_entry_drag(&mut self, mouse: MouseEvent) {
        let source_index = self.dragging_entry.take().unwrap();
        let target_index = self.entry_drag_target.take();
        let _ = mouse;

        match target_index {
            Some(t) if t != source_index => self.move_entry(source_index, t),
            _ => {
                self.explorer_selected = source_index;
                self.open_selected_entry();
            }
        }
    }

    fn move_entry(&mut self, source_index: usize, target_index:usize) {
        let source = match self.explorer_entries.get(source_index).cloned() {
            Some(e) => e,
            None => return,
        };

        let target_dir = match self.explorer_entries.get(target_index) {
            Some(e) if e.is_dir => e.path.clone(),
            Some(e) => match e.path.parent() {
                Some(p) => p.to_path_buf(),
                None => return,
            }
            None => return,
        };

        if source.is_dir && target_dir.starts_with(&source.path) {
            self.show_status("cannot move a folder into itself".to_string());
        }

        let file_name = match source.path.file_name() {
            Some(n) => n,
            None => return,
        };

        let dest = target_dir.join(file_name);

        if dest == source.path { return; }

        match std::fs::rename(&source.path, &dest) {
            Ok(()) => {
                if self.current_file == source.path.to_string_lossy() {
                    self.current_file = dest.to_string_lossy().to_string();
                }
                for tab in self.tabs_list.iter_mut() {
                    if tab.path == source.path {
                        tab.path = dest.clone();
                    }
                }
                self.refresh_explorer();
                self.show_status(format!("move to {}", dest.display()));
            }
            Err(_)=>self.show_status("failed to move file".to_string()),
        }
    }


    pub fn refresh_explorer(&mut self) {
        self.explorer_entries = Self::read_dir_sorted(&self.explorer_cwd, 0);
        self.explorer_selected = 0;
    }

    pub fn open_selected_entry(&mut self) {
        let entry = match self
            .explorer_entries
            .get(self.explorer_selected)
            .cloned()
        {
            Some(e) => e,
            None => return,
        };

        if entry.is_dir {
            if entry.expanded {
                self.collapse_entry(self.explorer_selected);
            } else {
                self.expand_entry(self.explorer_selected);
            } 
        } else {
            self.push_file_to_tabs(&entry.path);

        }
    }

    fn expand_entry(&mut self, index: usize) {
        let (path, depth) = {
            let entry = &self.explorer_entries[index];
            (entry.path.clone(), entry.depth)
        };

        let children = Self::read_dir_sorted(&path, depth+1);

        self.explorer_entries[index].expanded = true;
        self.explorer_entries.splice(index+1..index+1, children);
    }

    fn collapse_entry(&mut self, index: usize) {
        let depth = self.explorer_entries[index].depth;

        let end = self.explorer_entries[index + 1..]
            .iter()
            .position(|e| e.depth <= depth)
            .map(|offset| index + 1 + offset)
            .unwrap_or(self.explorer_entries.len());

        self.explorer_entries.drain(index + 1..end);
        self.explorer_entries[index].expanded = false;

        self.explorer_selected = self.explorer_selected.min(self.explorer_entries.len().saturating_sub(1));
    }

    pub fn icon_for(path: &Path, is_dir: bool) -> &str {
        if is_dir {
            return "󰉋";
        }

        match path.extension().and_then(|e| e.to_str()) {
            Some("rs") => "󱘗",
            Some("toml") => "",
            Some("json") => "",
            Some("md") => "󰍔",
            Some("txt") => "󰈙",
            Some("png") | Some("jpg") | Some("jpeg") => "󰈟",
            Some("svg") => "󰜡",
            Some("lock") => "󰌾",
            Some("gitignore") => "",
            _ => "󰈔",
        }
    }

    pub fn toggle_explorer(&mut self) {
        if self.show_explorer == false {
            self.show_explorer = true;
            self.refresh_explorer();
            self.show_status("Opened Explorer".to_string());
        } else {
            self.show_explorer = false;
            self.show_status("Closed Explorer".to_string());
        }
    }


    pub fn read_dir_sorted(path: &PathBuf, depth: usize) -> Vec<FileEntry> {
        let mut entries: Vec<FileEntry> = match std::fs::read_dir(path) {
            Ok(read) => read
                .filter_map(|e| e.ok())
                .map(|e| {
                    let path = e.path();
                    let name = e.file_name().to_string_lossy().to_string();
                    let is_dir = path.is_dir();
                    FileEntry { name, path, is_dir, depth, expanded: false}
                }).collect(),
            Err(_) => Vec::new(),
        };
        entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _=>a.name.to_lowercase().cmp(&b.name.to_lowercase()),        
        });

        entries
    }

}