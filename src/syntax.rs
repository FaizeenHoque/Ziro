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

impl Highlighter {
    pub fn new() -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
        }
    }

    pub fn highlight_lines(&self, content: &str, file_path: &str) -> Vec<Line<'static>> {
        let syntax = self
            .syntax_set
            .find_syntax_by_extension(file_path.split('.').last().unwrap_or("txt"))
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());

        let theme = &self.theme_set.themes["Solarized (dark)"];
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
                            text.to_string(), // FIX: own the string
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