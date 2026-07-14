use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

use crate::{app::App, ui::centered_rect};

impl App {
    pub fn filename_prompt(&self, area: ratatui::layout::Rect, buf: &mut Buffer) {
        let popup_area = centered_rect(40, 5, area);

        Clear.render(popup_area, buf);

        Block::new()
            .title(" Enter filename ")
            .borders(Borders::ALL)
            .style(
                Style::new()
                    .bg(self.theme.bg_popup)
                    .fg(self.theme.fg_default),
            )
            .render(popup_area, buf);

        let inner = Rect {
            x: popup_area.x + 1,
            y: popup_area.y + 2,
            width: popup_area.width - 2,
            height: 1,
        };

        Paragraph::new(self.filename_input.as_str())
            .style(
                Style::new()
                    .bg(self.theme.bg_popup)
                    .fg(self.theme.fg_default),
            )
            .render(inner, buf);
    }
}
