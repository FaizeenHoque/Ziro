use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Constraint, Layout},
    text::{Line, Span},
    style::Style,
    widgets::{Paragraph, Widget},
};

use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

use crate::app::App;

pub fn draw(frame: &mut Frame, app: &mut App) {
    frame.render_widget(app, frame.area());
}

fn highlight_lines<'a>(
    content: &str,
    file_path: &str,
    syntax_set: &SyntaxSet,
    theme_set: &ThemeSet,
) -> Vec<Line<'static>> {
    let syntax = syntax_set
        .find_syntax_by_extension(file_path.split('.').last().unwrap_or("txt"))
        .unwrap_or_else(|| syntax_set.find_syntax_plain_text());

    let theme = &theme_set.themes["Solarized (dark)"];
    let mut highlighter = HighlightLines::new(syntax, theme);

    LinesWithEndings::from(content)
        .map(|line| {
            let ranges = highlighter
                .highlight_line(line, syntax_set)
                .unwrap_or_default();

            let spans: Vec<Span<'static>> = ranges
                .into_iter()
                .map(|(style, text)| {
                    Span::styled(
                        text.to_string(), // FIX: owned string, no lifetime issue
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

impl Widget for &mut App {
    fn render(self, area: ratatui::layout::Rect, buf: &mut Buffer) {
        let chunks = Layout::vertical([
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(area);

        let viewport_height = chunks[0].height as usize;
        self.viewport_height.set(viewport_height);

        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme_set = ThemeSet::load_defaults();
        let content = self.document.lines.join("\n");

        let highlighted = highlight_lines(
            &content,
            &self.current_file,
            &syntax_set,
            &theme_set,
        );

        let text: Vec<Line> = highlighted
            .into_iter()
            .skip(self.scroll_y)
            .take(viewport_height)
            .enumerate()
            .map(|(i, line)| {
                let mut spans = vec![
                    Span::raw(format!("{:4} ", self.scroll_y + i + 1)),
                ];
                spans.extend(line.spans);
                Line::from(spans)
            })
            .collect();

        Paragraph::new(text).render(chunks[0], buf);

        let status = match self.mode {
            crate::mode::Mode::Insert if self.warning => {
                format!("< INSERT > {}", self.status_text)
            }
            crate::mode::Mode::Insert => "< INSERT >".to_string(),
            crate::mode::Mode::Command => self.command_input.clone(),
        };

        Paragraph::new(status).render(chunks[1], buf);

        self.reset_warning();
    }
}