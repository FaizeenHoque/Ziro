use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

use crate::{app::App, ui::rect_at_mouse};

pub struct ContextMenu {
    pub option: Vec<ContextOption>,
}
pub struct ContextOption {
    pub name: String,
    pub operation: fn(&mut App),
}

impl ContextOption {
    pub fn new(name: String, operation: fn(&mut App)) -> Self {
        Self { name, operation }
    }

    pub fn perform(&self, app: &mut App) {
        (self.operation)(app);
    }
}

impl ContextMenu {
    pub fn new(option: Vec<ContextOption>) -> Self {
        Self { option }
    }

    pub fn render(app: &App, area: Rect, buf: &mut Buffer, menu: &ContextMenu) {
        let Some((x, y)) = app.context_menu_pos else {
            return;
        };

        let content_height = menu.option.len() as u16;
        let popup_height = content_height + 2; // +2 for top/bottom border
        let popup_area = rect_at_mouse(x, y, 40, popup_height, area);

        Clear.render(popup_area, buf);

        let block = Block::new()
            .title(" Options:  ")
            .borders(Borders::ALL)
            .style(Style::new().bg(app.theme.bg_popup).fg(app.theme.fg_default));

        let inner = block.inner(popup_area);
        block.render(popup_area, buf);

        for (i, option) in menu.option.iter().enumerate() {
            let row = Rect {
                x: inner.x + 1,
                y: inner.y + i as u16,
                width: inner.width,
                height: 1,
            };
            Paragraph::new(option.name.as_str()).render(row, buf);
        }
    }
}
