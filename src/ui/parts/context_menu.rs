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

    fn popup_area(&self, app: &App, area: Rect) -> Option<Rect> {
        let Some((x, y)) = app.context_menu_pos else {
            return None;
        };

        let content_height = self.option.len() as u16;
        let popup_height = content_height + 2; // +2 for top/bottom border
        Some(rect_at_mouse(x, y, 40, popup_height, area))
    }

    pub fn render(app: &App, area: Rect, buf: &mut Buffer, menu: &ContextMenu) {
        let Some(popup_area) = menu.popup_area(app, area) else {
            return;
        };

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

    pub fn activate(&self, app: &mut App, area: Rect, mouse_x: u16, mouse_y: u16) -> bool {
        let Some(popup_area) = self.popup_area(app, area) else {
            return false;
        };

        if mouse_x < popup_area.x || mouse_x >= popup_area.x + popup_area.width {
            return false;
        }
        if mouse_y < popup_area.y || mouse_y >= popup_area.y + popup_area.height {
            return false;
        }

        let inner = Rect {
            x: popup_area.x + 1,
            y: popup_area.y + 1,
            width: popup_area.width.saturating_sub(2),
            height: popup_area.height.saturating_sub(2),
        };

        let option_index = mouse_y.saturating_sub(inner.y) as usize;
        if option_index >= self.option.len() {
            return false;
        }

        let Some(option) = self.option.get(option_index) else {
            return false;
        };

        option.perform(app);
        true
    }
}
