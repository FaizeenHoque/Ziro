use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::{app::App, ui::render_language_overlays};

impl App {
    pub fn render_top_titlebar(&self, area: Rect, buf: &mut Buffer) {
        Block::default()
            .style(Style::new().bg(self.theme.bg_titlebar))
            .render(area, buf);

        let top_bar = Paragraph::new(format!("Ziro @ {} ", self.current_file)).style(
            Style::new()
                .bg(self.theme.bg_titlebar)
                .fg(self.theme.fg_muted),
        );
        top_bar.render(area, buf);
    }

    pub fn render_bottom_statusbar(&self, area: Rect, buf: &mut Buffer) {
        Block::default()
            .style(Style::new().bg(self.theme.bg_statusbar))
            .render(area, buf);

        Paragraph::new(format!(
            " {}  Ln {}, Col {}  {}  LSP: {}  {}",
            self.status_text,
            self.cursor.y + 1,
            self.cursor.x + 1,
            self.language_name(),
            self.lsp_status_text(),
            if self.is_dirty() { "modified" } else { "saved" }
        ))
        .style(
            Style::new()
                .bg(self.theme.bg_statusbar)
                .fg(self.theme.fg_statusbar),
        )
        .render(area, buf);
    }

    pub fn render_explorer(&self, area: Rect, buf: &mut Buffer) {
        Block::new()
            .title("Explorer")
            .borders(Borders::ALL)
            .style(
                Style::new()
                    .bg(self.theme.bg_sidebar)
                    .fg(self.theme.fg_muted),
            )
            .render(area, buf);

        let inner = Rect {
            x: area.x + 2,
            y: area.y + 1,
            width: area.width.saturating_sub(3),
            height: area.height.saturating_sub(2),
        };

        self.explorer_area.set(inner);

        let lines: Vec<Line> = self
            .explorer_entries
            .iter()
            .enumerate()
            .map(|(i, entry)| {
                let selected = i == self.explorer_selected;
                let is_drag_target =
                    self.dragging_entry.is_some() && self.entry_drag_target == Some(i);
                let style = if is_drag_target {
                    Style::new()
                        .bg(self.theme.selected_bg)
                        .fg(self.theme.fg_accent)
                } else if selected {
                    Style::new().bg(self.theme.selected_bg).fg(Color::White)
                } else if entry.is_dir {
                    Style::new().bg(self.theme.bg_sidebar).fg(self.theme.fg_dir)
                } else {
                    Style::new()
                        .bg(self.theme.bg_sidebar)
                        .fg(self.theme.fg_default)
                };

                let indent = "  ".repeat(entry.depth);
                let marker = if entry.is_dir {
                    if entry.expanded { "v " } else { "> " }
                } else {
                    "  "
                };

                let icon = App::icon_for(&entry.path, entry.is_dir);
                let label = format!("{indent}{marker}{icon} {}", entry.name);
                Line::from(Span::styled(label, style))
            })
            .collect();

        Paragraph::new(lines)
            .style(Style::new().bg(self.theme.bg_sidebar))
            .render(inner, buf);
    }

    pub fn render_editor(&self, area: Rect, buf: &mut Buffer) {
        let horizontal = Layout::horizontal([
            Constraint::Length(self.number_col_width),
            Constraint::Min(1),
        ])
        .split(area);

        let viewport_height = area.height as usize;
        self.viewport_height.set(viewport_height);

        let content = self.document.lines.join("\n");
        let highlighted = self
            .highlighter
            .highlight_lines(&content, &self.current_file);

        // Render Line Numbers
        let numbers: Vec<Line> = highlighted
            .iter()
            .skip(self.scroll_y)
            .take(viewport_height)
            .enumerate()
            .map(|(i, _line)| {
                let line_no = self.scroll_y + i;
                let is_current = line_no == self.cursor.y;
                let fg = if is_current {
                    self.theme.fg_line_number_active
                } else {
                    self.theme.fg_line_number
                };
                Line::from(vec![Span::styled(
                    format!("{:>4} ", line_no + 1),
                    Style::new().fg(fg).bg(self.theme.bg_editor),
                )])
            })
            .collect();

        Paragraph::new(numbers)
            .style(Style::new().bg(self.theme.bg_editor))
            .render(horizontal[0], buf);

        // Render Highlighted Content Code
        let content_lines: Vec<Line> = highlighted
            .into_iter()
            .skip(self.scroll_y)
            .take(viewport_height)
            .map(|line| Line::from(line.spans))
            .collect();

        Paragraph::new(content_lines)
            .style(
                Style::new()
                    .bg(self.theme.bg_editor)
                    .fg(self.theme.fg_default),
            )
            .render(horizontal[1], buf);

        // Call LSP layers to draw on top of content bounds
        render_language_overlays(self, horizontal[1], buf);
    }
}
