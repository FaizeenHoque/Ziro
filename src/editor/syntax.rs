use ratatui::{style::{Color, Modifier, Style}, text::{Line, Span}};
use syntect::{easy::HighlightLines, highlighting::{FontStyle, Theme, ThemeSet}, parsing::SyntaxSet, util::LinesWithEndings};

// Embedded at COMPILE time, not read from disk at runtime.
// This means the theme travels inside the binary — works on any machine
// you ship this to, and can never fail from a missing/wrong path.
// Put the actual file at: assets/OneDark.tmTheme (relative to this source file's crate root)
const ONE_DARK_THEME_BYTES: &[u8] = include_bytes!("../../assets/OneDark.tmTheme");

pub struct Highlighter {
    pub syntax_set: SyntaxSet,
    pub theme_set: ThemeSet,
    pub active_theme: String,
}

impl std::fmt::Debug for Highlighter {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("Highlighter").field("active_theme", &self.active_theme).finish()
    }
}

impl Highlighter {
    pub fn new() -> Self {
        let mut theme_set = ThemeSet::load_defaults();

        // Loud failure. If this theme doesn't load, you find out at startup,
        // not by squinting at your editor wondering why it's still muted.
        // Silent fallback here is exactly what hid the bug last time.
        let one_dark: Theme = load_embedded_theme(ONE_DARK_THEME_BYTES)
            .expect("bundled One Dark theme failed to parse — check assets/OneDark.tmTheme is a valid .tmTheme XML file");
        theme_set.themes.insert("one-dark".to_string(), one_dark);

        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set,
            active_theme: "one-dark".to_string(),
        }
    }

    pub fn set_theme(&mut self, name: impl Into<String>) -> bool {
        let name = name.into();
        if !self.theme_set.themes.contains_key(&name) { return false; }
        self.active_theme = name;
        true
    }

    pub fn highlight_lines(&self, content: &str, file_path: &str) -> Vec<Line<'static>> {
        if content.is_empty() { return vec![Line::from("")]; }

        let syntax = self.syntax_for(file_path);

        // Same loud-fallback principle: if active_theme somehow doesn't exist
        // (e.g. someone called set_theme with a bad name and ignored the bool),
        // fall back explicitly and know about it, rather than silently guessing.
        let theme = self.theme_set.themes.get(&self.active_theme)
            .unwrap_or_else(|| {
                debug_assert!(false, "active_theme '{}' not found in theme_set, falling back to base16-ocean.dark", self.active_theme);
                &self.theme_set.themes["base16-ocean.dark"]
            });

        let mut highlighter = HighlightLines::new(syntax, theme);

        LinesWithEndings::from(content)
            .map(|line| {
                // THIS is the fix for "code disappearing." Previously:
                //   .unwrap_or_default()
                // silently turned a highlight error into an EMPTY Vec<Span>,
                // which means the line rendered as blank — the text itself
                // was gone, not just uncolored. Never let a highlighting
                // failure delete the underlying text.
                let spans: Vec<Span<'static>> = match highlighter.highlight_line(line, &self.syntax_set) {
                    Ok(ranges) => ranges
                        .into_iter()
                        .map(|(style, text)| {
                            let mut output = Style::new().fg(Color::Rgb(
                                style.foreground.r,
                                style.foreground.g,
                                style.foreground.b,
                            ));
                            if style.font_style.contains(FontStyle::BOLD) {
                                output = output.add_modifier(Modifier::BOLD);
                            }
                            if style.font_style.contains(FontStyle::ITALIC) {
                                output = output.add_modifier(Modifier::ITALIC);
                            }
                            Span::styled(text.to_string(), output)
                        })
                        .collect(),
                    Err(_) => vec![Span::raw(line.to_string())],
                };
                Line::from(spans)
            })
            .collect()
    }

    fn syntax_for(&self, file_path: &str) -> &syntect::parsing::SyntaxReference {
        std::path::Path::new(file_path)
            .extension()
            .and_then(|extension| extension.to_str())
            .and_then(|extension| self.syntax_set.find_syntax_by_extension(extension))
            .or_else(|| file_path.is_empty().then(|| self.syntax_set.find_syntax_by_token("Rust")).flatten())
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text())
    }
}

fn load_embedded_theme(bytes: &[u8]) -> Result<Theme, syntect::LoadingError> {
    let mut reader = std::io::Cursor::new(bytes);
    ThemeSet::load_from_reader(&mut reader)
}