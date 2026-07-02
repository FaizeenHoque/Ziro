use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    text::{Line, Span},
    style::{Style, Color},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

use crate::app::{App, EXPLORER_WIDTH};

// ---- VS Code Dark+ palette -------------------------------------------------
// const BG_EDITOR: Color = Color::Rgb(30, 30, 30); // #1e1e1e
// const BG_SIDEBAR: Color = Color::Rgb(37, 37, 38); // #252526
// const BG_TITLEBAR: Color = Color::Rgb(51, 51, 51); // #333333
// const BG_TABBAR: Color = Color::Rgb(37, 37, 38); // #252526
// const BG_TAB_ACTIVE: Color = Color::Rgb(30, 30, 30); // #1e1e1e (merges into editor)
// const BG_TAB_INACTIVE: Color = Color::Rgb(45, 45, 45); // #2d2d2d
// const BG_STATUSBAR: Color = Color::Rgb(0, 122, 204); // #007acc
// const BG_POPUP: Color = Color::Rgb(37, 37, 38); // #252526

// const FG_DEFAULT: Color = Color::Rgb(212, 212, 212); // #d4d4d4
// const FG_MUTED: Color = Color::Rgb(153, 153, 153); // #999999
// const FG_STATUSBAR: Color = Color::White;
// const FG_LINE_NUMBER: Color = Color::Rgb(133, 133, 133); // #858585
// const FG_LINE_NUMBER_ACTIVE: Color = Color::White;
// const FG_DIR: Color = Color::Rgb(197, 148, 106); // #c5946a (folder tan, vscode-ish)
// const FG_ACCENT: Color = Color::Rgb(86, 156, 214); // #569cd6

// const SELECTED_BG: Color = Color::Rgb(38, 79, 120); // #264f78 (list.activeSelectionBackground)


// ---- Zed "One Dark" palette ------------------------------------------------
// const BG_EDITOR: Color = Color::Rgb(40, 44, 52);      // #282c34
// const BG_SIDEBAR: Color = Color::Rgb(33, 37, 43);     // #21252b
// const BG_TITLEBAR: Color = Color::Rgb(33, 37, 43);    // #21252b
// const BG_TABBAR: Color = Color::Rgb(33, 37, 43);      // #21252b
// const BG_TAB_ACTIVE: Color = Color::Rgb(40, 44, 52);  // #282c34 (merges into editor)
// const BG_TAB_INACTIVE: Color = Color::Rgb(33, 37, 43);// #21252b
// const BG_STATUSBAR: Color = Color::Rgb(33, 37, 43);   // #21252b (Zed keeps this quiet, not a loud accent bar)
// const BG_POPUP: Color = Color::Rgb(33, 37, 43);       // #21252b

// const FG_DEFAULT: Color = Color::Rgb(171, 178, 191);  // #abb2bf
// const FG_MUTED: Color = Color::Rgb(92, 99, 112);       // #5c6370
// const FG_STATUSBAR: Color = Color::Rgb(171, 178, 191); // #abb2bf
// const FG_LINE_NUMBER: Color = Color::Rgb(73, 81, 98);  // #495162
// const FG_LINE_NUMBER_ACTIVE: Color = Color::Rgb(171, 178, 191); // #abb2bf
// const FG_DIR: Color = Color::Rgb(229, 192, 123);       // #e5c07b (Zed's folder/keyword gold)
// const FG_ACCENT: Color = Color::Rgb(97, 175, 239);     // #61afef (Zed blue)

// const SELECTED_BG: Color = Color::Rgb(62, 68, 81);     // #3e4451

// ---- Matte Black palette ---------------------------------------------------
const BG_EDITOR: Color = Color::Rgb(18, 18, 18);       // #121212
const BG_SIDEBAR: Color = Color::Rgb(22, 22, 22);      // #161616
const BG_TITLEBAR: Color = Color::Rgb(26, 26, 26);     // #1a1a1a
const BG_TABBAR: Color = Color::Rgb(22, 22, 22);       // #161616
const BG_TAB_ACTIVE: Color = Color::Rgb(18, 18, 18);   // #121212 (merges into editor)
const BG_TAB_INACTIVE: Color = Color::Rgb(30, 30, 30); // #1e1e1e
const BG_STATUSBAR: Color = Color::Rgb(26, 26, 26);    // #1a1a1a (flat, no color pop)
const BG_POPUP: Color = Color::Rgb(22, 22, 22);        // #161616

const FG_DEFAULT: Color = Color::Rgb(224, 224, 224);   // #e0e0e0
const FG_MUTED: Color = Color::Rgb(110, 110, 110);     // #6e6e6e
const FG_STATUSBAR: Color = Color::Rgb(224, 224, 224); // #e0e0e0
const FG_LINE_NUMBER: Color = Color::Rgb(74, 74, 74);  // #4a4a4a
const FG_LINE_NUMBER_ACTIVE: Color = Color::Rgb(208, 208, 208); // #d0d0d0
const FG_DIR: Color = Color::Rgb(201, 168, 118);       // #c9a876 (muted brass, only touch of color)
const FG_ACCENT: Color = Color::Rgb(160, 160, 160);    // #a0a0a0 (grayscale accent, no blue)

const SELECTED_BG: Color = Color::Rgb(42, 42, 42);     // #2a2a2a

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

fn render_explorer(app: &App, area: Rect, buf: &mut Buffer) {
    Block::new()
        .title("Explorer")
        .borders(Borders::ALL)
        .style(Style::new().bg(BG_SIDEBAR).fg(FG_MUTED))
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
        let style = if selected {
            Style::new().bg(SELECTED_BG).fg(Color::White)
        } else if entry.is_dir {
            Style::new().bg(BG_SIDEBAR).fg(FG_DIR)
        } else {
            Style::new().bg(BG_SIDEBAR).fg(FG_DEFAULT)
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
        .style(Style::new().bg(BG_SIDEBAR))
        .render(inner, buf);
}

fn render_tabs(app: &App, area: Rect, buf: &mut Buffer) {
    Block::new().style(Style::new().bg(BG_TABBAR)).render(area, buf);
    app.tabs_area.set(area);

    let mut x = area.x;
    for tab in &app.tabs_list {
        let active = tab.path.to_string_lossy() == app.current_file;
        let style = if active {
            Style::new().bg(BG_TAB_ACTIVE).fg(Color::White)
        } else {
            Style::new().bg(BG_TAB_INACTIVE).fg(FG_MUTED)
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
                    cell.set_style(Style::new().bg(BG_TAB_ACTIVE).fg(FG_ACCENT));
                }
            }
        }
        x += width;
    }

    if x < area.x + area.width {
        Paragraph::new("")
            .style(Style::new().bg(BG_TABBAR))
            .render(Rect { x, y: area.y, width: area.x + area.width - x, height: 1 }, buf);
    }
}

impl Widget for &mut App {
    fn render(self, area: ratatui::layout::Rect, buf: &mut Buffer) {

        Block::new()
            .style(Style::new().bg(BG_EDITOR).fg(FG_DEFAULT))
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
                let fg = if is_current { FG_LINE_NUMBER_ACTIVE } else { FG_LINE_NUMBER };
                Line::from(vec![
                    Span::styled(
                        format!("{:>4} ", line_no + 1),
                        Style::new().fg(fg).bg(BG_EDITOR),
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
        Block::default().style(Style::new().bg(BG_TITLEBAR)).render(top_bar_area, buf);
        let top_bar = Paragraph::new(format!("Ziro @ {} ", self.current_file))
            .style(Style::new().bg(BG_TITLEBAR).fg(FG_MUTED));
        top_bar.render(top_bar_area, buf);

        // STATUS BAR
        Block::default().style(Style::new().bg(BG_STATUSBAR)).render(status_area, buf);
        Paragraph::new(format!(" STATUS {}", self.status_text))
            .style(Style::new().bg(BG_STATUSBAR).fg(FG_STATUSBAR))
            .render(status_area, buf);

        Paragraph::new(numbers)
            .style(Style::new().bg(BG_EDITOR))
            .render(horizontal[0], buf);
        Paragraph::new(content_lines)
            .style(Style::new().bg(BG_EDITOR).fg(FG_DEFAULT))
            .render(horizontal[1], buf);

        // Filename popup
        if self.filename_prompt {
            let popup_area = centered_rect(40, 5, area);

            Clear.render(popup_area, buf);

            Block::new()
                .title(" Enter filename ")
                .borders(Borders::ALL)
                .style(Style::new().bg(BG_POPUP).fg(FG_DEFAULT))
                .render(popup_area, buf);

            let inner = Rect {
                x: popup_area.x + 1,
                y: popup_area.y + 2,
                width: popup_area.width - 2,
                height: 1,
            };

            Paragraph::new(self.filename_input.as_str())
                .style(Style::new().bg(BG_POPUP).fg(FG_DEFAULT))
                .render(inner, buf);
        }
    }
}