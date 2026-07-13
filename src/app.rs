use std::path::Path;
use std::{io, path::PathBuf};
use std::cell::Cell;

use ratatui::layout::Rect;
use ratatui::{
    DefaultTerminal,
    Frame,
};

use crate::lsp::{LspClient, LspMessage};
use crate::management::{TabItem, UndoState};
use crate::ui::Theme;
use crate::{
    editor::*, ui,
};


#[derive(Debug)]
pub struct App {
    pub document: Document,
    pub theme: Theme,
    pub lsp_messages: Vec<LspMessage>,
    pub lsp: Option<LspClient>,
    pub cursor: Cursor,
    pub highlighter: Highlighter,

    pub undo_stack: Vec<UndoState>,
    pub redo_stack: Vec<UndoState>,
    pub last_action: ActionKind,
    pub last_saved: Vec<String>,

    pub scroll_y: usize,
    pub viewport_height: Cell<usize>,
    pub number_col_width: u16,
    pub editor_area: Cell<Rect>,

    pub current_file: String,
    pub filename_input: String,

    pub status_text: String,
    pub status: bool,
    pub filename_prompt: bool,
    pub show_explorer: bool,

    pub explorer_area: Cell<Rect>,
    pub explorer_entries: Vec<FileEntry>,
    pub explorer_selected: usize,
    pub explorer_cwd: PathBuf,

    pub tabs_area: Cell<Rect>,
    pub tabs_list: Vec<TabItem>,

    pub dragging_tab: Option<usize>,
    pub tab_drag_target: Option<usize>,

    pub dragging_entry: Option<usize>,
    pub entry_drag_target: Option<usize>,

    pub pending_quit_after_save: bool,
    pub exit: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActionKind {
    None,
    Insert,
    Delete,
    Newline,
}

impl Default for App {
    fn default() -> Self {
        let document = Document::default();
        let last_saved = document.lines.clone();
        Self {
            document,
            theme: Theme::by_name("matte"),
            lsp: LspClient::new().ok(),
            lsp_messages: Vec::new(),

            current_file: String::new(),
            highlighter: Highlighter::new(),
            cursor: Cursor::default(),

            viewport_height: Cell::new(20),
            number_col_width: 7,
            scroll_y: 0,
            editor_area: Cell::new(Rect::default()),

            filename_input: String::new(),
            status_text: String::new(),

            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            last_action: ActionKind::None,
            last_saved,

            filename_prompt: false,
            show_explorer: false,

            explorer_area: Cell::new(Rect::default()),
            explorer_entries: Vec::new(),
            explorer_selected: 0,
            explorer_cwd: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),

            tabs_area: Cell::new(Rect::default()),
            tabs_list: Vec::new(),

            dragging_tab: None,
            tab_drag_target: None,

            dragging_entry: None,
            entry_drag_target: None,

            pending_quit_after_save: false,
            status: false,
            exit: false,
        }
    }
}

pub const EXPLORER_WIDTH: u16 = 40;

impl App {
    pub fn new(file: Option<String>) -> io::Result<Self> {
        let mut app = Self::default();
        if let Some(filename) = file {
            app.push_file_to_tabs(Path::new(&filename));
        }
        if let Some(lsp) = app.lsp.as_mut() {
            lsp.initialize()?;
            lsp.initialized()?;
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
            let ea = self.editor_area.get();
            let x = ea.x + self.number_col_width + self.cursor.x as u16;
            let y = ea.y + (self.cursor.y - self.scroll_y) as u16;

            frame.set_cursor_position((
                x.min(area.width.saturating_sub(1)),
                y.min(area.height.saturating_sub(2)),
            ));
        }
    }

    pub fn point_in_rect(col: u16, row: u16, area: Rect) -> bool {
        col >= area.x && col < area.x + area.width
            && row >= area.y && row < area.y + area.height
    }

    pub fn editor_x_offset(&self) -> u16 {
        if self.show_explorer { EXPLORER_WIDTH } else { 0 }
    }

    pub fn is_dirty(&self) -> bool {
        self.document.lines != self.last_saved
    }

    pub fn switch_to_file(&mut self, tab: TabItem) {
        self.save_current_tab_state();

        match Document::from_file(tab.path.to_str().unwrap_or_default()) {
            Ok(doc) => {
                self.document = doc;
                self.current_file = tab.path.to_string_lossy().to_string();
                self.last_saved = self.document.lines.clone();
                self.cursor.x = tab.cursor.x;
                self.cursor.y = tab.cursor.y;
                self.scroll_y = tab.scroll_y;
                let path = std::path::PathBuf::from(&self.current_file);
                let text = self.document.lines.join("\n");

                if let Some(lsp) = self.lsp.as_mut() {
                    if let Err(e) = lsp.did_open(&path, &text) {
                        crate::debug::log(format!("did_open failed: {e}"));
                    }
                }
            }

            Err(_) => {
                self.show_status("failed to open file".to_string());
            }
        }
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
}
