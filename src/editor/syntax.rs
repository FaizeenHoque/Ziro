use ratatui::{style::{Color, Modifier, Style}, text::{Line, Span}};
use syntect::{easy::HighlightLines, highlighting::{FontStyle, Theme, ThemeSet}, parsing::SyntaxSet, util::LinesWithEndings};

// Embedded at COMPILE time, not read from disk at runtime.
// This means the theme travels inside the binary — works on any machine
// you ship this to, and can never fail from a missing/wrong path.
// Put the actual file at: assets/ZiroDark.tmTheme (relative to this source file's crate root)
const ZIRO_DARK_THEME_BYTES: &[u8] = include_bytes!("../../assets/ZiroDark.tmTheme");

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


        let ziro_dark: Theme = load_embedded_theme(ZIRO_DARK_THEME_BYTES)
            .expect("bundled Ziro Dark theme failed to parse — check assets/Dracula.tmTheme is a valid .tmTheme XML file");
        theme_set.themes.insert("ziro-dark".to_string(), ziro_dark);

        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set,
            active_theme: "ziro-dark".to_string(),
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
        let theme = self.theme_set.themes.get(&self.active_theme)
            .unwrap_or_else(|| {
                debug_assert!(false, "active_theme '{}' not found in theme_set, falling back to base16-ocean.dark", self.active_theme);
                &self.theme_set.themes["base16-ocean.dark"]
            });

        let mut highlighter = HighlightLines::new(syntax, theme);

        LinesWithEndings::from(content)
            .map(|line| {
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