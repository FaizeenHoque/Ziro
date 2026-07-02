use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    text::{Line, Span},
    style::{Style, Color},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

use crate::app::App;

pub fn draw(frame: &mut Frame, app: &mut App) {
    frame.render_widget(app, frame.area());
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    Rect {
        x: area.width.saturating_sub(width) / 2,
        y: area.height.saturating_sub(height) / 2,
        width: width.min(area.width),
        height: height.min(area.height),
    }
}

impl Widget for &mut App {
    fn render(self, area: ratatui::layout::Rect, buf: &mut Buffer) {

        let bar_style = Style::default().bg(Color::Rgb(20, 20, 20)).fg(Color::White);

        // Text editor area
        let chunks = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ]).split(area);

        let horizontal = Layout::horizontal([
            Constraint::Length(self.number_col_width),
            Constraint::Min(1),
        ]).split(chunks[1]);


        let viewport_height = chunks[1].height as usize;
        self.viewport_height.set(viewport_height);

        let content = self.document.lines.join("\n");

        let highlighted = self.highlighter.highlight_lines(&content, &self.current_file);

        let numbers: Vec<Line> = highlighted
            .iter()
            .skip(self.scroll_y)
            .take(viewport_height)
            .enumerate()
            .map(|(i, _line)| {
                Line::from(vec![
                    Span::styled(
                        format!("{:>4} ", self.scroll_y + i + 1),
                        Style::new().fg(Color::LightBlue),
                    ),
                ])
            })
            .collect();   

        let content_lines: Vec<Line> = highlighted
            .into_iter()
            .skip(self.scroll_y)
            .take(viewport_height)
            .map(|line| Line::from(line.spans))
            .collect();

        let top_bar = Paragraph::new(format!(" {} ", self.current_file))
            .style(Style::new().fg(Color::DarkGray));
        top_bar.render(chunks[0], buf);

        Block::default()
            .style(bar_style)
            .render(chunks[0], buf);

        Paragraph::new(numbers).render(horizontal[0], buf);
        Paragraph::new(content_lines).render(horizontal[1], buf);

        let status = format!("STATUS {}", self.status_text);
        Block::default()
            .style(bar_style)
            .render(chunks[2], buf);
        Paragraph::new(status).render(chunks[2], buf);

        // Filename popup
        if self.filename_prompt == true {
            let popup_area = centered_rect(40, 5, area);

            Clear.render(popup_area, buf);

            Block::new()
                .title(" Enter filename ")
                .borders(Borders::ALL)
                .render(popup_area, buf);

            let inner = Rect {
                x: popup_area.x + 1,
                y: popup_area.y + 2,
                width: popup_area.width - 2,
                height: 1,
            };

            Paragraph::new(self.filename_input.as_str())
                .render(inner, buf);
        }
    }
}