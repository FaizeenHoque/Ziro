use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Style,
    widgets::{Block, Paragraph, Widget},
};

use crate::{
    app::{App, EXPLORER_WIDTH},
    ui::*,
};

pub fn draw(frame: &mut Frame, app: &mut App) {
    frame.render_widget(app, frame.area());
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Clear background
        Block::new()
            .style(
                Style::new()
                    .bg(self.theme.bg_editor)
                    .fg(self.theme.fg_default),
            )
            .render(area, buf);

        // Core Layout splitting
        let outer = Layout::vertical([
            Constraint::Length(1), // Title Bar
            Constraint::Min(1),    // Body Area
            Constraint::Length(1), // Status Bar
        ])
        .split(area);

        let (top_bar_area, body_area, status_area) = (outer[0], outer[1], outer[2]);

        // Render top title bar and bottom status bar (implementations below!)
        self.render_top_titlebar(top_bar_area, buf);
        self.render_bottom_statusbar(status_area, buf);

        // Calculate sidebar columns
        let editor_col = if self.show_explorer {
            let split =
                Layout::horizontal([Constraint::Length(EXPLORER_WIDTH), Constraint::Min(1)])
                    .split(body_area);

            self.render_explorer(split[0], buf);
            split[1]
        } else {
            body_area
        };

        let editor_split =
            Layout::vertical([Constraint::Length(1), Constraint::Min(1)]).split(editor_col);

        // Render Tabs if open
        if !self.tabs_list.is_empty() {
            render_tabs(self, editor_split[0], buf);
        }

        let editor_area = editor_split[1];
        self.editor_area.set(editor_area);

        // Render Code Lines & Line Numbers
        self.render_editor(editor_area, buf);

        // Render Filename Prompts/Overlays
        if self.filename_prompt {
            self.filename_prompt(area, buf);
        }
    }
}
