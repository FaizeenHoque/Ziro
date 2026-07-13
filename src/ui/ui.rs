use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    text::{Line, Span},
    style::{Style, Color, Modifier},
    widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap},
};

use crate::{app::{App, EXPLORER_WIDTH}, ui::*};

pub fn draw(frame: &mut Frame, app: &mut App) {
    frame.render_widget(app, frame.area());
}

impl Widget for &mut App {
    fn render(self, area: ratatui::layout::Rect, buf: &mut Buffer) {

        Block::new()
            .style(Style::new().bg(self.theme.bg_editor).fg(self.theme.fg_default))
            .render(area, buf);

        // Top bar / editor / status bar split
        let outer = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ]).split(area);

        let (top_bar_area, body_area, status_area) = (outer[0], outer[1], outer[2]);

        let editor_col = if self.show_explorer {
            let split = Layout::horizontal([
                Constraint::Length(EXPLORER_WIDTH),
                Constraint::Min(1),
            ]).split(body_area);

            render_explorer(self, split[0], buf);
            split[1]
        } else {
            body_area
        };

        let editor_split = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(1),
        ]).split(editor_col);

        if self.tabs_list.len() > 0 {
            render_tabs(self, editor_split[0], buf);
        }

        let editor_area = editor_split[1];
        self.editor_area.set(editor_area);

        let horizontal = Layout::horizontal([
            Constraint::Length(self.number_col_width),
            Constraint::Min(1),
        ]).split(editor_area);

        let viewport_height = editor_area.height as usize;
        self.viewport_height.set(viewport_height);

        let content = self.document.lines.join("\n");
        let highlighted = self.highlighter.highlight_lines(&content, &self.current_file);

        let numbers: Vec<Line> = highlighted
            .iter()
            .skip(self.scroll_y)
            .take(viewport_height)
            .enumerate()
            .map(|(i, _line)| {
                let line_no = self.scroll_y + i;
                let is_current = line_no == self.cursor.y;
                let fg = if is_current { self.theme.fg_line_number_active } else { self.theme.fg_line_number };
                Line::from(vec![
                    Span::styled(
                        format!("{:>4} ", line_no + 1),
                        Style::new().fg(fg).bg(self.theme.bg_editor),
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

        // TOP BAR
        Block::default().style(Style::new().bg(self.theme.bg_titlebar)).render(top_bar_area, buf);
        let top_bar = Paragraph::new(format!("Ziro @ {} ", self.current_file))
            .style(Style::new().bg(self.theme.bg_titlebar).fg(self.theme.fg_muted));
        top_bar.render(top_bar_area, buf);

        // STATUS BAR
        Block::default().style(Style::new().bg(self.theme.bg_statusbar)).render(status_area, buf);
        Paragraph::new(format!(" {}  Ln {}, Col {}  {}  LSP: {}  {}", self.status_text, self.cursor.y + 1, self.cursor.x + 1, self.language_name(), if self.lsp_ready { "alive" } else { "offline" }, if self.is_dirty() { "modified" } else { "saved" }))
            .style(Style::new().bg(self.theme.bg_statusbar).fg(self.theme.fg_statusbar))
            .render(status_area, buf);

        Paragraph::new(numbers)
            .style(Style::new().bg(self.theme.bg_editor))
            .render(horizontal[0], buf);
        Paragraph::new(content_lines)
            .style(Style::new().bg(self.theme.bg_editor).fg(self.theme.fg_default))
            .render(horizontal[1], buf);
        render_language_overlays(self, horizontal[1], buf);

        // Filename popup
        if self.filename_prompt {
            let popup_area = centered_rect(40, 5, area);

            Clear.render(popup_area, buf);

            Block::new()
                .title(" Enter filename ")
                .borders(Borders::ALL)
                .style(Style::new().bg(self.theme.bg_popup).fg(self.theme.fg_default))
                .render(popup_area, buf);

            let inner = Rect {
                x: popup_area.x + 1,
                y: popup_area.y + 2,
                width: popup_area.width - 2,
                height: 1,
            };

            Paragraph::new(self.filename_input.as_str())
                .style(Style::new().bg(self.theme.bg_popup).fg(self.theme.fg_default))
                .render(inner, buf);
        }
    }
}

fn render_language_overlays(app: &App, area: Rect, buf: &mut Buffer) {
    render_semantic_tokens(app, area, buf);
    for diagnostic in &app.diagnostics {
        let line = diagnostic.range.start.line as usize;
        if line < app.scroll_y || line >= app.scroll_y + area.height as usize { continue; }
        let text = &app.document.lines[line];
        let start = utf16_column(text, diagnostic.range.start.character as usize);
        let end = utf16_column(text, diagnostic.range.end.character as usize).max(start + 1);
        let color = if diagnostic.severity == Some(2) { Color::Yellow } else { Color::Red };
        let y = area.y + (line - app.scroll_y) as u16;
        for x in start..end.min(text.chars().count()) { if let Some(cell) = buf.cell_mut((area.x + x as u16, y)) { cell.set_style(Style::new().fg(color).add_modifier(Modifier::UNDERLINED)); } }
        if start >= text.chars().count() { if let Some(cell) = buf.cell_mut((area.x + start.min(area.width.saturating_sub(1) as usize) as u16, y)) { cell.set_symbol("^").set_style(Style::new().fg(color)); } }
    }
    if !app.completions.is_empty() { let page = app.completion_selected / 8 * 8; let visible = &app.completions[page..app.completions.len().min(page + 8)]; let width = visible.iter().map(|item| item.label.len()).max().unwrap_or(0).min(50) as u16 + 4; let height = visible.len() as u16; let x = (area.x + app.cursor.x as u16).min(area.x + area.width.saturating_sub(width)); let y = (area.y + (app.cursor.y - app.scroll_y) as u16 + 1).min(area.y + area.height.saturating_sub(height)); let popup = Rect { x, y, width, height }; Clear.render(popup, buf); for (index, item) in visible.iter().enumerate() { let style = if page + index == app.completion_selected { Style::new().bg(app.theme.selected_bg).fg(Color::White) } else { Style::new().bg(app.theme.bg_popup).fg(app.theme.fg_default) }; Paragraph::new(format!(" {} {}", completion_icon(item.kind), item.label)).style(style).render(Rect { x, y: y + index as u16, width, height: 1 }, buf); } return; }
    let text = app.hover.as_ref().map(|hover| hover_text(&hover.contents)).or_else(|| app.hover_position.and_then(|(line, character)| app.diagnostics.iter().find(|diagnostic| (line, character) >= (diagnostic.range.start.line as usize, diagnostic.range.start.character as usize) && (line, character) <= (diagnostic.range.end.line as usize, diagnostic.range.end.character as usize)).map(|diagnostic| format!("{}", diagnostic.message)))).unwrap_or_default();
    if text.is_empty() { return; }
    let (anchor_x, anchor_y) = app.hover_anchor.unwrap_or((area.x, area.y)); let right_space = (area.x + area.width).saturating_sub(anchor_x + 1); let left_space = anchor_x.saturating_sub(area.x); let width = right_space.max(left_space).min(60); if width == 0 { return; } let height = text.lines().map(|line| line.chars().count().max(1).div_ceil(width as usize)).sum::<usize>().clamp(1, 6) as u16; let x = if right_space >= left_space { anchor_x + 1 } else { anchor_x - width }; let below_space = (area.y + area.height).saturating_sub(anchor_y + 1); let above_space = anchor_y.saturating_sub(area.y); let y = if below_space >= height || below_space >= above_space { anchor_y + 1 } else { anchor_y.saturating_sub(height) }; let popup = Rect { x, y, width, height: height.min(area.y + area.height - y) }; Clear.render(popup, buf); Paragraph::new(text).wrap(Wrap { trim: true }).style(Style::new().bg(app.theme.bg_popup).fg(app.theme.fg_default)).render(popup, buf);
}

fn render_semantic_tokens(app: &App, area: Rect, buf: &mut Buffer) {
    for (line, tokens) in &app.semantic_tokens {
        if *line < app.scroll_y || *line >= app.scroll_y + area.height as usize { continue; }
        let Some(text) = app.document.lines.get(*line) else { continue; };
        let y = area.y + (*line - app.scroll_y) as u16;
        for token in tokens {
            let color = match token.token_type.as_str() {
                "namespace" | "type" | "class" | "enum" | "interface" | "struct" | "typeParameter" => Color::Rgb(86, 182, 194),
                "function" | "method" => Color::Rgb(97, 175, 239),
                "property" | "enumMember" => Color::Rgb(224, 108, 117),
                "parameter" | "variable" => Color::Rgb(224, 108, 117),
                "macro" => Color::Rgb(198, 120, 221),
                _ => continue,
            };
            let start = utf16_column(text, token.start);
            let end = utf16_column(text, token.start + token.length);
            for x in start..end { if let Some(cell) = buf.cell_mut((area.x + x as u16, y)) { cell.set_style(Style::new().fg(color)); } }
        }
    }
}

fn utf16_column(text: &str, offset: usize) -> usize { let mut units = 0; for (column, character) in text.chars().enumerate() { if units >= offset { return column; } units += character.len_utf16(); } text.chars().count() }
fn hover_text(value: &serde_json::Value) -> String { match value { serde_json::Value::String(text) => text.clone(), serde_json::Value::Array(items) => items.iter().map(hover_text).collect::<Vec<_>>().join("\n"), serde_json::Value::Object(object) => object.get("value").and_then(serde_json::Value::as_str).unwrap_or_default().to_string(), _ => String::new() } }

fn completion_icon(kind: Option<u32>) -> &'static str { match kind { Some(1) => "T", Some(2 | 3 | 4) => "ƒ", Some(5 | 6 | 7 | 8 | 9 | 10) => "◇", Some(14) => "▣", Some(15 | 16 | 17 | 18 | 19 | 20) => "•", Some(21 | 22) => "↗", Some(23) => "⌘", Some(24) => "▰", _ => "·" } }

fn render_explorer(app: &App, area: Rect, buf: &mut Buffer) {
    Block::new()
        .title("Explorer")
        .borders(Borders::ALL)
        .style(Style::new().bg(app.theme.bg_sidebar).fg(app.theme.fg_muted))
        .render(area, buf);

    let inner = Rect {
        x: area.x + 2,
        y: area.y + 1,
        width: area.width.saturating_sub(3),
        height: area.height.saturating_sub(2),
    };

    app.explorer_area.set(inner);

    let lines: Vec<Line> = app.explorer_entries.iter().enumerate().map(|(i, entry)| {
        let selected = i == app.explorer_selected;
        let is_drag_target = app.dragging_entry.is_some() && app.entry_drag_target == Some(i);
        let style = if is_drag_target {
            Style::new().bg(app.theme.selected_bg).fg(app.theme.fg_accent)
        } else if selected {
            Style::new().bg(app.theme.selected_bg).fg(Color::White)
        } else if entry.is_dir {
            Style::new().bg(app.theme.bg_sidebar).fg(app.theme.fg_dir)
        } else {
            Style::new().bg(app.theme.bg_sidebar).fg(app.theme.fg_default)
        };

        let indent = "  ".repeat(entry.depth);
        let marker = if entry.is_dir {
            if entry.expanded { "v " } else { "> " }
        } else {
            "  "
        };

        let icon = App::icon_for(&entry.path, entry.is_dir);
        let label = format!(
            "{indent}{marker}{icon} {}",
            entry.name
        );
        Line::from(Span::styled(label, style))
    }).collect();

    Paragraph::new(lines)
        .style(Style::new().bg(app.theme.bg_sidebar))
        .render(inner, buf);
}

fn render_tabs(app: &App, area: Rect, buf: &mut Buffer) {
    Block::new().style(Style::new().bg(app.theme.bg_tabbar)).render(area, buf);
    app.tabs_area.set(area);

    let mut x = area.x;
    for (i, tab) in app.tabs_list.iter().enumerate() {
        let active = tab.path.to_string_lossy() == app.current_file;
        let is_drag_target = app.dragging_tab.is_some() && app.tab_drag_target == Some(i);
        let style = if is_drag_target {
            Style::new().bg(app.theme.selected_bg).fg(app.theme.fg_accent)
        } else if active {
            Style::new().bg(app.theme.bg_tab_active).fg(Color::White)
        } else {
            Style::new().bg(app.theme.bg_tab_inactive).fg(app.theme.fg_muted)
        };
        let label = format!(" {} x ", tab.name);
        let width = label.len() as u16;
        if x + width > area.x + area.width { break; }

        Paragraph::new(label).style(style).render(
            Rect { x, y: area.y, width, height: 1 }, buf
        );
        if active {
            for cx in x..x + width {
                if let Some(cell) = buf.cell_mut((cx, area.y)) {
                    cell.set_style(Style::new().bg(app.theme.bg_tab_active).fg(app.theme.fg_accent));
                }
            }
        }
        x += width;
    }

    if x < area.x + area.width {
        Paragraph::new("")
            .style(Style::new().bg(app.theme.bg_tabbar))
            .render(Rect { x, y: area.y, width: area.x + area.width - x, height: 1 }, buf);
    }
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    Rect {
        x: area.width.saturating_sub(width) / 2,
        y: area.height.saturating_sub(height) / 2,
        width: width.min(area.width),
        height: height.min(area.height),
    }
}
