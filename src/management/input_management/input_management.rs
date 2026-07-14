use crossterm::event::{
    self, Event, KeyCode, KeyEventKind, MouseButton, MouseEvent, MouseEventKind,
};
use std::io;
use std::time::Duration;

use crate::app::{ActionKind, App};
use crate::lsp::{LspMessage, LspStatus};

impl App {
    pub fn handle_events(&mut self) -> io::Result<()> {
        if self.poll_and_dispatch(Duration::from_millis(16))? {
            while self.poll_and_dispatch(Duration::from_millis(0))? {}
        }

        self.poll_hover();
        self.poll_semantic_tokens();

        let current_language = self.language_id();
        let languages: Vec<String> = self.lsp_sessions.keys().cloned().collect();

        for language in languages {
            let is_current = current_language == Some(language.as_str());
            loop {
                let message = match self.lsp_sessions.get(&language) {
                    Some(session) => session.client.try_recv(),
                    None => None,
                };
                let Some(message) = message else {
                    break;
                };

                match message {
                    LspMessage::Initialized(legend) => {
                        if let Some(session) = self.lsp_sessions.get_mut(&language) {
                            let _ = session.client.initialized();
                            session.semantic_legend = legend;
                            session.status = LspStatus::Alive;
                        }
                        if is_current {
                            self.open_current_document();
                        }
                    }
                    LspMessage::Completion(mut items) if is_current => {
                        let prefix = self.document.lines[self.cursor.y][..self.cursor.x]
                            .rsplit(|character: char| {
                                !character.is_alphanumeric() && character != '_'
                            })
                            .next()
                            .unwrap_or_default()
                            .to_lowercase();
                        if !prefix.is_empty() {
                            items.retain(|item| item.label.to_lowercase().starts_with(&prefix));
                        }
                        self.completions = items;
                        self.completion_selected = 0;
                    }
                    LspMessage::Diagnostics(uri, items)
                        if is_current
                            && uri
                                == crate::lsp::protocol::path_to_uri(std::path::Path::new(
                                    &self.current_file,
                                )) =>
                    {
                        self.diagnostics = items;
                    }
                    LspMessage::Hover(line, character, hover)
                        if is_current && self.hover_position == Some((line, character)) =>
                    {
                        self.hover = hover;
                    }
                    LspMessage::SemanticTokens(data) if is_current => {
                        let legend = self
                            .lsp_sessions
                            .get(&language)
                            .map(|s| s.semantic_legend.clone())
                            .unwrap_or_default();
                        self.set_semantic_tokens(&legend, data);
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    /// Polls for a single terminal event within `timeout` and dispatches it.
    /// Returns Ok(true) if an event was found and handled.
    fn poll_and_dispatch(&mut self, timeout: Duration) -> io::Result<bool> {
        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    self.handle_key(key);
                }
                Event::Mouse(mouse) => {
                    self.handle_mouse(mouse);
                }
                _ => {}
            }
            return Ok(true);
        }
        Ok(false)
    }

    pub fn accept_completion(&mut self) {
        let Some(item) = self.completions.get(self.completion_selected) else {
            return;
        };
        let mut text = item
            .insert_text
            .clone()
            .unwrap_or_else(|| item.label.clone());
        let mut cursor = text.len();
        if matches!(item.kind, Some(2..=4)) {
            if !text.contains('(') {
                text.push_str("()");
            }
            cursor = text.find('(').map_or(text.len(), |index| index + 1);
        }
        let start = self.document.lines[self.cursor.y][..self.cursor.x]
            .rfind(|c: char| !c.is_alphanumeric() && c != '_')
            .map_or(0, |index| index + 1);
        self.document.lines[self.cursor.y].replace_range(start..self.cursor.x, &text);
        self.cursor.x = start + cursor;
        self.completions.clear();
        self.document_changed(false);
    }
}
