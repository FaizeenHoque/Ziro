use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Paragraph, Widget},
};

use crate::app::App;

pub fn render_tabs(app: &App, area: Rect, buf: &mut Buffer) {
    Block::new()
        .style(Style::new().bg(app.theme.bg_tabbar))
        .render(area, buf);
    app.tabs_area.set(area);

    let mut x = area.x;
    for (i, tab) in app.tabs_list.iter().enumerate() {
        let active = tab.path.to_string_lossy() == app.current_file;
        let is_drag_target = app.dragging_tab.is_some() && app.tab_drag_target == Some(i);
        let style = if is_drag_target {
            Style::new()
                .bg(app.theme.selected_bg)
                .fg(app.theme.fg_accent)
        } else if active {
            Style::new().bg(app.theme.bg_tab_active).fg(Color::White)
        } else {
            Style::new()
                .bg(app.theme.bg_tab_inactive)
                .fg(app.theme.fg_muted)
        };
        let label = format!(" {} x ", tab.name);
        let width = label.len() as u16;
        if x + width > area.x + area.width {
            break;
        }

        Paragraph::new(label).style(style).render(
            Rect {
                x,
                y: area.y,
                width,
                height: 1,
            },
            buf,
        );
        if active {
            for cx in x..x + width {
                if let Some(cell) = buf.cell_mut((cx, area.y)) {
                    cell.set_style(
                        Style::new()
                            .bg(app.theme.bg_tab_active)
                            .fg(app.theme.fg_accent),
                    );
                }
            }
        }
        x += width;
    }

    if x < area.x + area.width {
        Paragraph::new("")
            .style(Style::new().bg(app.theme.bg_tabbar))
            .render(
                Rect {
                    x,
                    y: area.y,
                    width: area.x + area.width - x,
                    height: 1,
                },
                buf,
            );
    }
}
