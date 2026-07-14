use ratatui::layout::Rect;

pub fn utf16_column(text: &str, offset: usize) -> usize {
    let mut units = 0;
    for (column, character) in text.chars().enumerate() {
        if units >= offset {
            return column;
        }
        units += character.len_utf16();
    }
    text.chars().count()
}

pub fn hover_text(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(text) => text.clone(),
        serde_json::Value::Array(items) => {
            items.iter().map(hover_text).collect::<Vec<_>>().join("\n")
        }
        serde_json::Value::Object(object) => object
            .get("value")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string(),
        _ => String::new(),
    }
}

pub fn completion_icon(kind: Option<u32>) -> &'static str {
    match kind {
        Some(1) => "T",
        Some(2 | 3 | 4) => "ƒ",
        Some(5 | 6 | 7 | 8 | 9 | 10) => "◇",
        Some(14) => "▣",
        Some(15 | 16 | 17 | 18 | 19 | 20) => "•",
        Some(21 | 22) => "↗",
        Some(23) => "⌘",
        Some(24) => "▰",
        _ => "·",
    }
}

pub fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    Rect {
        x: area.width.saturating_sub(width) / 2,
        y: area.height.saturating_sub(height) / 2,
        width: width.min(area.width),
        height: height.min(area.height),
    }
}
