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
            .enumerate()
            .map(|(i, line)| {
                let line_no = self.scroll_y + i;
                let spans = if let Some(selection) = &self.selection {
                    let (start, end) = selection.range((self.cursor.x, self.cursor.y));
                    if line_no >= start.1 && line_no <= end.1 {
                        let sel_start = if line_no == start.1 { start.0 } else { 0 };
                        let sel_end = if line_no == end.1 {
                            end.0
                        } else {
                            self.document.lines[line_no].chars().count()
                        };
                        Self::apply_selection_highlight(
                            line.spans,
                            sel_start,
                            sel_end,
                            self.theme.selected_bg,
                        )
                    } else {
                        line.spans
                    }
                } else {
                    line.spans
                };
                Line::from(spans)
            })
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

    pub fn apply_selection_highlight<'a>(
        spans: Vec<Span<'a>>,
        sel_start: usize,
        sel_end: usize,
        selection_bg: Color,
    ) -> Vec<Span<'a>> {
        if sel_start >= sel_end {
            return spans;
        }

        let mut result = Vec::new();
        let mut col = 0usize;

        for span in spans {
            let text = span.content.to_string();
            let len = text.chars().count();
            let span_start = col;
            let span_end = col + len;
            col = span_end;

            // No overlap with selection — keep span as-is.
            if span_end <= sel_start || span_start >= sel_end {
                result.push(Span::styled(text, span.style));
                continue;
            }

            // Overlap exists — split into up to three parts: before, selected, after.
            let local_sel_start = sel_start.saturating_sub(span_start).min(len);
            let local_sel_end = sel_end.saturating_sub(span_start).min(len);

            let chars: Vec<char> = text.chars().collect();
            let before: String = chars[..local_sel_start].iter().collect();
            let selected: String = chars[local_sel_start..local_sel_end].iter().collect();
            let after: String = chars[local_sel_end..].iter().collect();

            if !before.is_empty() {
                result.push(Span::styled(before, span.style));
            }
            if !selected.is_empty() {
                result.push(Span::styled(selected, span.style.bg(selection_bg)));
            }
            if !after.is_empty() {
                result.push(Span::styled(after, span.style));
            }
        }

        result
    }
}
