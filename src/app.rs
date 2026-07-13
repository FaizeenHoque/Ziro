use std::{collections::{BTreeMap, HashMap}, path::Path, time::{Duration, Instant}};
use std::{io, path::PathBuf};
use std::cell::Cell;

use ratatui::layout::Rect;
use ratatui::{
    DefaultTerminal,
    Frame,
};

use crate::lsp::{LspClient, LspSession, LspStatus};
use crate::lsp::protocol::{CompletionItem, Diagnostic, Hover, SemanticToken};
use crate::management::{TabItem, UndoState};
use crate::ui::Theme;
use crate::{
    editor::*, ui,
};


#[derive(Debug)]
pub struct App {
    pub document: Document,
    pub theme: Theme,
    pub lsp_sessions: HashMap<String, LspSession>,
    pub semantic_tokens: BTreeMap<usize, Vec<SemanticToken>>,
    pub semantic_refresh: Option<Instant>,
    pub diagnostics: Vec<Diagnostic>, pub completions: Vec<CompletionItem>, pub completion_selected: usize, pub hover: Option<Hover>, pub hover_position: Option<(usize, usize)>, pub hover_anchor: Option<(u16, u16)>, pub hover_pending: Option<(usize, usize, u16, u16, Instant)>,
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
            lsp_sessions: HashMap::new(),
            semantic_tokens: BTreeMap::new(), semantic_refresh: None, diagnostics: Vec::new(), completions: Vec::new(), completion_selected: 0, hover: None, hover_position: None, hover_anchor: None, hover_pending: None,

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
        if let Some(filename) = file { app.push_file_to_tabs(Path::new(&filename)); }
        Ok(app)
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal,) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| {
                self.draw(frame);
            })?;

            self.handle_events()?;
        }

        self.shutdown_lsp_sessions();
        Ok(())
    }

    fn shutdown_lsp_sessions(&mut self) {
        for session in self.lsp_sessions.values_mut() {
            let _ = session.client.shutdown();
        }
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

    pub fn language_name(&self) -> &'static str {
        match Path::new(&self.current_file).extension().and_then(|extension| extension.to_str()) {
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

    pub fn current_session_mut(&mut self) -> Option<&mut LspSession> {
        let language = self.language_id()?;
        self.lsp_sessions.get_mut(language)
    }

    pub fn current_session(&self) -> Option<&LspSession> {
        let language = self.language_id()?;
        self.lsp_sessions.get(language)
    }

    /// Lazily spawns a session for the current file's language if one
    /// doesn't exist yet. Does nothing if no server is installed —
    /// editing stays syntect-only, never blocked.
    pub fn ensure_lsp_session(&mut self) {
        let Some(language) = self.language_id() else { return; };
        if self.lsp_sessions.contains_key(language) { return; }

        let Some((binary, args)) = crate::lsp::registry::find_server(language) else {
            return;
        };

        match LspClient::spawn(&binary, args, language) {
            Ok(mut client) => {
                let start = Path::new(&self.current_file).parent()
                    .map(Path::to_path_buf)
                    .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
                let root = crate::lsp::registry::find_root(&start, language);
                let _ = client.initialize(&root);
                self.lsp_sessions.insert(language.to_string(), LspSession {
                    client,
                    status: LspStatus::Initializing,
                    semantic_legend: Vec::new(),
                });
            }
            Err(_) => {
                // Binary was found on PATH but failed to spawn (permissions,
                // corrupt install, etc). No session inserted — same
                // "none found" treatment rather than a half-broken entry.
            }
        }
    }

    pub fn lsp_status_text(&self) -> String {
        match self.language_id() {
            None => "n/a".to_string(),
            Some(language) => match self.lsp_sessions.get(language) {
                Some(session) => format!("{language} {}", match session.status {
                    LspStatus::Initializing => "initializing",
                    LspStatus::Alive => "alive",
                    LspStatus::Offline => "offline",
                }),
                None => format!("{language}: none found"),
            }
        }
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

    pub fn document_changed(&mut self, completion: bool) {
        self.hover = None;
        self.hover_pending = None;
        self.hover_position = None;
        self.completions.clear();
        self.diagnostics.clear();
        self.semantic_tokens.clear();
        self.semantic_refresh = Some(Instant::now());

        let Some(path) = (!self.current_file.is_empty()).then(|| PathBuf::from(&self.current_file)) else {
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
        if self.current_file.is_empty() { return; }
        let is_alive = self.current_session().map(|s| s.status == LspStatus::Alive).unwrap_or(false);
        if !is_alive { return; }
        let path = Path::new(&self.current_file).to_path_buf();
        let text = self.document.lines.join("\n");
        if let Some(session) = self.current_session_mut() {
            let _ = session.client.did_open(&path, &text);
            let _ = session.client.semantic_tokens(&path);
        }
    }

    pub fn set_semantic_tokens(&mut self, legend: &[String], data: Vec<u32>) {
        self.semantic_tokens.clear();
        let (mut line, mut start) = (0usize, 0usize);
        for chunk in data.chunks_exact(5) {
            line += chunk[0] as usize;
            start = if chunk[0] == 0 { start + chunk[1] as usize } else { chunk[1] as usize };
            if let Some(token_type) = legend.get(chunk[3] as usize).cloned() {
                self.semantic_tokens.entry(line).or_default().push(SemanticToken { start, length: chunk[2] as usize, token_type, modifiers: chunk[4] });
            }
        }
    }

    pub fn poll_semantic_tokens(&mut self) {
        if self.semantic_refresh.is_none_or(|time| time.elapsed() < Duration::from_millis(250)) { return; }
        self.semantic_refresh = None;
        let path = Path::new(&self.current_file).to_path_buf();
        if let Some(session) = self.current_session_mut() {
            let _ = session.client.semantic_tokens(&path);
        }
    }

    pub fn request_hover_at(&mut self, column: u16, row: u16) {
        let area = self.editor_area.get(); let x = area.x + self.number_col_width;
        if row < area.y || row >= area.y + area.height || column < x { self.hover_pending = None; self.hover = None; return; }
        let line = self.scroll_y + (row - area.y) as usize;
        let Some(text) = self.document.lines.get(line) else { return; };
        let offset = (column - x) as usize;
        let Some((index, character)) = text.char_indices().nth(offset) else { self.hover_pending = None; self.hover = None; return; };
        if !character.is_alphanumeric() && character != '_' { self.hover_pending = None; self.hover = None; return; }
        let position = text[..index].encode_utf16().count();
        self.hover = None;
        self.hover_pending = Some((line, position, column, row, Instant::now()));
    }

    pub fn poll_hover(&mut self) {
        let Some((line, character, x, y, since)) = self.hover_pending else { return; };
        if since.elapsed() < Duration::from_millis(300) { return; }
        self.hover_pending = None;
        self.hover_position = Some((line, character));
        self.hover_anchor = Some((x, y));
        let path = Path::new(&self.current_file).to_path_buf();
        if let Some(session) = self.current_session_mut() {
            let _ = session.client.hover(&path, line, character);
        }
    }
}