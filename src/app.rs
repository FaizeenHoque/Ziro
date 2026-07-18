use std::cell::Cell;
use std::{
    collections::{BTreeMap, HashMap},
    path::Path,
    time::{Duration, Instant},
};
use std::{io, path::PathBuf};

use ratatui::layout::Rect;
use ratatui::{DefaultTerminal, Frame};

use crate::lsp::protocol::{CompletionItem, Diagnostic, Hover, SemanticToken};
use crate::lsp::{LspSession, LspStatus};
use crate::management::{Selection, TabItem, UndoState};
use crate::ui::Theme;
use crate::{editor::*, ui};

pub struct App {
    pub document: Document,
    pub theme: Theme,
    pub lsp_sessions: HashMap<String, LspSession>,
    pub semantic_tokens: BTreeMap<usize, Vec<SemanticToken>>,
    pub semantic_refresh: Option<Instant>,
    pub diagnostics: Vec<Diagnostic>,
    pub completions: Vec<CompletionItem>,
    pub completion_selected: usize,
    pub hover: Option<Hover>,
    pub hover_position: Option<(usize, usize)>,
    pub hover_anchor: Option<(u16, u16)>,
    pub hover_pending: Option<(usize, usize, u16, u16, Instant)>,
    pub cursor: Cursor,
    pub highlighter: Highlighter,
    pub last_line_count: usize,
    pub clipboard: Option<arboard::Clipboard>,
    pub selection: Option<Selection>,
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

impl std::fmt::Debug for App {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("App")
            .field("document", &self.document)
            .field("current_file", &self.current_file)
            .field("cursor", &self.cursor)
            .field("clipboard", &self.clipboard.is_some())
            .field("undo_stack_len", &self.undo_stack.len())
            .field("last_action", &self.last_action)
            .finish_non_exhaustive()
    }
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
        let last_line_count = document.lines.len();

        Self {
            document,
            theme: Theme::by_name("matte"),
            lsp_sessions: HashMap::new(),
            semantic_tokens: BTreeMap::new(),
            semantic_refresh: None,
            diagnostics: Vec::new(),
            completions: Vec::new(),
            completion_selected: 0,
            hover: None,
            hover_position: None,
            hover_anchor: None,
            hover_pending: None,
            last_line_count,

            clipboard: arboard::Clipboard::new().ok(),
            selection: None,

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
        Ok(app)
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| {
                self.draw(frame);
            })?;

            self.handle_events()?;
        }

        self.shutdown_lsp_sessions();
        Ok(())
    }

    // misc
    fn draw(&mut self, frame: &mut Frame) {
        ui::draw(frame, self);
        let area = frame.area();

        if self.filename_prompt == true {
            let popup_x = area.width.saturating_sub(40) / 2;
            let popup_y = area.height.saturating_sub(5) / 2;
            frame
                .set_cursor_position((popup_x + 1 + self.filename_input.len() as u16, popup_y + 2));
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
        col >= area.x && col < area.x + area.width && row >= area.y && row < area.y + area.height
    }

    pub fn editor_x_offset(&self) -> u16 {
        if self.show_explorer {
            EXPLORER_WIDTH
        } else {
            0
        }
    }

    // input
    pub fn update_scroll(&mut self, viewport_height: usize) {
        if self.cursor.y < self.scroll_y {
            self.scroll_y = self.cursor.y;
        }

        if self.cursor.y >= self.scroll_y + viewport_height {
            self.scroll_y = self.cursor.y - viewport_height + 1;
        }
    }

    pub fn request_hover_at(&mut self, column: u16, row: u16) {
        let area = self.editor_area.get();
        let x = area.x + self.number_col_width;
        if row < area.y || row >= area.y + area.height || column < x {
            self.hover_pending = None;
            self.hover = None;
            return;
        }
        let line = self.scroll_y + (row - area.y) as usize;
        let Some(text) = self.document.lines.get(line) else {
            return;
        };
        let offset = (column - x) as usize;
        let Some((index, character)) = text.char_indices().nth(offset) else {
            self.hover_pending = None;
            self.hover = None;
            return;
        };
        if !character.is_alphanumeric() && character != '_' {
            self.hover_pending = None;
            self.hover = None;
            return;
        }
        let position = text[..index].encode_utf16().count();
        self.hover = None;
        self.hover_pending = Some((line, position, column, row, Instant::now()));
    }

    pub fn poll_hover(&mut self) {
        let Some((line, character, x, y, since)) = self.hover_pending else {
            return;
        };
        if since.elapsed() < Duration::from_millis(300) {
            return;
        }
        self.hover_pending = None;
        self.hover_position = Some((line, character));
        self.hover_anchor = Some((x, y));
        let path = Path::new(&self.current_file).to_path_buf();
        if let Some(session) = self.current_session_mut() {
            let _ = session.client.hover(&path, line, character);
        }
    }

    pub fn clamp_cursor_x(&mut self) {
        let line_len = self
            .document
            .lines
            .get(self.cursor.y)
            .map(|l| l.chars().count())
            .unwrap_or(0);
        self.cursor.x = self.cursor.x.min(line_len);
    }

    // status
    pub fn show_status(&mut self, text: String) {
        self.status = true;
        self.status_text = text;
    }

    pub fn reset_status(&mut self) {
        self.status = false;
        self.status_text = String::new();
    }

    // documents and files
    pub fn is_dirty(&self) -> bool {
        self.document.lines != self.last_saved
    }

    pub fn switch_to_file(&mut self, tab: TabItem) {
        self.save_current_tab_state();

        match Document::from_file(tab.path.to_str().unwrap_or_default()) {
            Ok(doc) => {
                self.document = doc;
                self.current_file = tab.path.to_string_lossy().to_string();
                self.diagnostics.clear();
                self.completions.clear();
                self.hover = None;
                self.hover_pending = None;
                self.hover_position = None;
                self.semantic_tokens.clear();
                self.semantic_refresh = None;
                self.last_saved = self.document.lines.clone();
                self.cursor.x = tab.cursor.x;
                self.cursor.y = tab.cursor.y;
                self.scroll_y = tab.scroll_y;
                self.ensure_lsp_session();
                self.open_current_document();
            }

            Err(_) => {
                self.show_status("failed to open file".to_string());
            }
        }
    }

    pub fn document_changed(&mut self, completion: bool) {
        self.hover = None;
        self.hover_pending = None;
        self.hover_position = None;
        self.completions.clear();

        let current_line_count = self.document.lines.len();
        if current_line_count != self.last_line_count {
            // Line count changed (Enter, or a backspace that merged two lines) —
            // every semantic token / diagnostic below the edit point now points
            // at the wrong line. Stale-but-same-shape data is fine to keep
            // briefly; stale-and-wrong-shape data is what produces garbled
            // highlighting, so this is the one case where clearing is correct.
            self.semantic_tokens.clear();
            self.diagnostics.clear();
            self.last_line_count = current_line_count;
        }

        self.semantic_refresh = Some(Instant::now());

        let Some(path) = (!self.current_file.is_empty()).then(|| PathBuf::from(&self.current_file))
        else {
            return;
        };

        let text = self.document.lines.join("\n");
        let line = self.cursor.y;
        let character = self.document.lines[line][..self.cursor.x]
            .encode_utf16()
            .count();

        if let Some(session) = self.current_session_mut() {
            let _ = session.client.did_change(&path, &text);

            if completion {
                let _ = session.client.completion(&path, line, character);
            }
        }
    }

    pub fn open_current_document(&mut self) {
        if self.current_file.is_empty() {
            return;
        }
        let is_alive = self
            .current_session()
            .map(|s| s.status == LspStatus::Alive)
            .unwrap_or(false);
        if !is_alive {
            return;
        }
        let path = Path::new(&self.current_file).to_path_buf();
        let text = self.document.lines.join("\n");
        if let Some(session) = self.current_session_mut() {
            let _ = session.client.did_open(&path, &text);
            let _ = session.client.semantic_tokens(&path);
        }
    }

    pub fn language_name(&self) -> &'static str {
        match Path::new(&self.current_file)
            .extension()
            .and_then(|extension| extension.to_str())
        {
            Some("rs") => "Rust",
            Some("py") => "Python",
            Some("js" | "jsx") => "JavaScript",
            Some("ts" | "tsx") => "TypeScript",
            Some("go") => "Go",
            Some("c" | "h") => "C",
            Some("cpp" | "cc" | "cxx" | "hpp") => "C++",
            Some("java") => "Java",
            Some("json") => "JSON",
            Some("toml") => "TOML",
            Some("md") => "Markdown",
            _ => "Plain Text",
        }
    }

    pub fn language_id(&self) -> Option<&'static str> {
        crate::lsp::registry::language_for_path(Path::new(&self.current_file))
    }
}
