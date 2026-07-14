use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Clear, Paragraph, Widget, Wrap},
};

use crate::{
    app::{App, EXPLORER_WIDTH},
    ui::*,
};

pub fn render_language_overlays(app: &App, area: Rect, buf: &mut Buffer) {
    render_semantic_tokens(app, area, buf);
    for diagnostic in &app.diagnostics {
        let line = diagnostic.range.start.line as usize;
        if line < app.scroll_y || line >= app.scroll_y + area.height as usize {
            continue;
        }
        let text = &app.document.lines[line];
        let start = utf16_column(text, diagnostic.range.start.character as usize);
        let end = utf16_column(text, diagnostic.range.end.character as usize).max(start + 1);
        let color = if diagnostic.severity == Some(2) {
            Color::Yellow
        } else {
            Color::Red
        };
        let y = area.y + (line - app.scroll_y) as u16;
        for x in start..end.min(text.chars().count()) {
            if let Some(cell) = buf.cell_mut((area.x + x as u16, y)) {
                cell.set_style(Style::new().fg(color).add_modifier(Modifier::UNDERLINED));
            }
        }
        if start >= text.chars().count() {
            if let Some(cell) = buf.cell_mut((
                area.x + start.min(area.width.saturating_sub(1) as usize) as u16,
                y,
            )) {
                cell.set_symbol("^").set_style(Style::new().fg(color));
            }
        }
    }
    if !app.completions.is_empty() {
        let page = app.completion_selected / 8 * 8;
        let visible = &app.completions[page..app.completions.len().min(page + 8)];
        let width = visible
            .iter()
            .map(|item| item.label.len())
            .max()
            .unwrap_or(0)
            .min(50) as u16
            + 4;
        let height = visible.len() as u16;
        let x = (area.x + app.cursor.x as u16).min(area.x + area.width.saturating_sub(width));
        let y = (area.y + (app.cursor.y - app.scroll_y) as u16 + 1)
            .min(area.y + area.height.saturating_sub(height));
        let popup = Rect {
            x,
            y,
            width,
            height,
        };
        Clear.render(popup, buf);
        for (index, item) in visible.iter().enumerate() {
            let style = if page + index == app.completion_selected {
                Style::new().bg(app.theme.selected_bg).fg(Color::White)
            } else {
                Style::new().bg(app.theme.bg_popup).fg(app.theme.fg_default)
            };
            Paragraph::new(format!(" {} {}", completion_icon(item.kind), item.label))
                .style(style)
                .render(
                    Rect {
                        x,
                        y: y + index as u16,
                        width,
                        height: 1,
                    },
                    buf,
                );
        }
        return;
    }
    let text = app
        .hover
        .as_ref()
        .map(|hover| hover_text(&hover.contents))
        .or_else(|| {
            app.hover_position.and_then(|(line, character)| {
                app.diagnostics
                    .iter()
                    .find(|diagnostic| {
                        (line, character)
                            >= (
                                diagnostic.range.start.line as usize,
                                diagnostic.range.start.character as usize,
                            )
                            && (line, character)
                                <= (
                                    diagnostic.range.end.line as usize,
                                    diagnostic.range.end.character as usize,
                                )
                    })
                    .map(|diagnostic| format!("{}", diagnostic.message))
            })
        })
        .unwrap_or_default();
    if text.is_empty() {
        return;
    }
    let (anchor_x, anchor_y) = app.hover_anchor.unwrap_or((area.x, area.y));
    let right_space = (area.x + area.width).saturating_sub(anchor_x + 1);
    let left_space = anchor_x.saturating_sub(area.x);
    let width = right_space.max(left_space).min(60);
    if width == 0 {
        return;
    }
    let height = text
        .lines()
        .map(|line| line.chars().count().max(1).div_ceil(width as usize))
        .sum::<usize>()
        .clamp(1, 6) as u16;
    let x = if right_space >= left_space {
        anchor_x + 1
    } else {
        anchor_x - width
    };
    let below_space = (area.y + area.height).saturating_sub(anchor_y + 1);
    let above_space = anchor_y.saturating_sub(area.y);
    let y = if below_space >= height || below_space >= above_space {
        anchor_y + 1
    } else {
        anchor_y.saturating_sub(height)
    };
    let popup = Rect {
        x,
        y,
        width,
        height: height.min(area.y + area.height - y),
    };
    Clear.render(popup, buf);
    Paragraph::new(text)
        .wrap(Wrap { trim: true })
        .style(Style::new().bg(app.theme.bg_popup).fg(app.theme.fg_default))
        .render(popup, buf);
}

pub fn render_semantic_tokens(app: &App, area: Rect, buf: &mut Buffer) {
    for (line, tokens) in &app.semantic_tokens {
        if *line < app.scroll_y || *line >= app.scroll_y + area.height as usize {
            continue;
        }
        let Some(text) = app.document.lines.get(*line) else {
            continue;
        };
        let y = area.y + (*line - app.scroll_y) as u16;
        for token in tokens {
            let color = match token.token_type.as_str() {
                "namespace" | "type" | "class" | "enum" | "interface" | "struct"
                | "typeParameter" => Color::Rgb(86, 182, 194),
                "function" | "method" => Color::Rgb(97, 175, 239),
                "property" | "enumMember" => Color::Rgb(224, 108, 117),
                "parameter" | "variable" => Color::Rgb(224, 108, 117),
                "macro" => Color::Rgb(198, 120, 221),
                _ => continue,
            };
            let start = utf16_column(text, token.start);
            let end = utf16_column(text, token.start + token.length);
            for x in start..end {
                if let Some(cell) = buf.cell_mut((area.x + x as u16, y)) {
                    cell.set_style(Style::new().fg(color));
                }
            }
        }
    }
}
