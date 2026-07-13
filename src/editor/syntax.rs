use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;
use ratatui::text::{Line, Span};
use ratatui::style::Style;

pub struct Highlighter {
    pub syntax_set: SyntaxSet,
    pub theme_set: ThemeSet,
}

// SyntaxSet/ThemeSet don't implement Debug, so this is hand-rolled purely
// to satisfy `#[derive(Debug)]` on App, which holds a Highlighter.
impl std::fmt::Debug for Highlighter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Highlighter").finish()
    }
}

impl Highlighter {
    pub fn new() -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
        }
    }

    pub fn highlight_lines(&self, content: &str, file_path: &str) -> Vec<Line<'static>> {

        if content.is_empty() {
            return vec![Line::from("")];
        }

        let syntax = self
            .syntax_set
            .find_syntax_by_extension(file_path.split('.').last().unwrap_or("txt"))
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());

        let theme = &self.theme_set.themes["base16-eighties.dark"];
        let mut highlighter = HighlightLines::new(syntax, theme);

        LinesWithEndings::from(content)
            .map(|line| {
                let ranges = highlighter
                    .highlight_line(line, &self.syntax_set)
                    .unwrap_or_default();

                let spans: Vec<Span<'static>> = ranges
                    .into_iter()
                    .map(|(style, text)| {
                        Span::styled(
                            text.to_string(), 
                            Style::new().fg(ratatui::style::Color::Rgb(
                                style.foreground.r,
                                style.foreground.g,
                                style.foreground.b,
                            )),
                        )
                    })
                    .collect();

                Line::from(spans)
            })
            .collect() 
    }
}