use ratatui::style::Color;

#[derive(Clone, Copy, Debug)]
pub struct Theme {
    pub bg_editor: Color,
    pub bg_sidebar: Color,
    pub bg_titlebar: Color,
    pub bg_tabbar: Color,
    pub bg_tab_active: Color,
    pub bg_tab_inactive: Color,
    pub bg_statusbar: Color,
    pub bg_popup: Color,
    pub fg_default: Color,
    pub fg_muted: Color,
    pub fg_statusbar: Color,
    pub fg_line_number: Color,
    pub fg_line_number_active: Color,
    pub fg_dir: Color,
    pub fg_accent: Color,
    pub selected_bg: Color,
}

pub const VSCODE_DARK_PLUS: Theme = Theme {
    bg_editor: Color::Rgb(30, 30, 30),
    bg_sidebar: Color::Rgb(37, 37, 38),
    bg_titlebar: Color::Rgb(51, 51, 51),
    bg_tabbar: Color::Rgb(37, 37, 38),
    bg_tab_active: Color::Rgb(30, 30, 30),
    bg_tab_inactive: Color::Rgb(45, 45, 45),
    bg_statusbar: Color::Rgb(0, 122, 204),
    bg_popup: Color::Rgb(37, 37, 38),
    fg_default: Color::Rgb(212, 212, 212),
    fg_muted: Color::Rgb(153, 153, 153),
    fg_statusbar: Color::White,
    fg_line_number: Color::Rgb(133, 133, 133),
    fg_line_number_active: Color::White,
    fg_dir: Color::Rgb(197, 148, 106),
    fg_accent: Color::Rgb(86, 156, 214),
    selected_bg: Color::Rgb(38, 79, 120),
};

pub const ZED_ONE_DARK: Theme = Theme {
    bg_editor: Color::Rgb(40, 44, 52),
    bg_sidebar: Color::Rgb(33, 37, 43),
    bg_titlebar: Color::Rgb(33, 37, 43),
    bg_tabbar: Color::Rgb(33, 37, 43),
    bg_tab_active: Color::Rgb(40, 44, 52),
    bg_tab_inactive: Color::Rgb(33, 37, 43),
    bg_statusbar: Color::Rgb(33, 37, 43),
    bg_popup: Color::Rgb(33, 37, 43),
    fg_default: Color::Rgb(171, 178, 191),
    fg_muted: Color::Rgb(92, 99, 112),
    fg_statusbar: Color::Rgb(171, 178, 191),
    fg_line_number: Color::Rgb(73, 81, 98),
    fg_line_number_active: Color::Rgb(171, 178, 191),
    fg_dir: Color::Rgb(229, 192, 123),
    fg_accent: Color::Rgb(97, 175, 239),
    selected_bg: Color::Rgb(62, 68, 81),
};

pub const MATTE_BLACK: Theme = Theme {
    bg_editor: Color::Rgb(18, 18, 18),
    bg_sidebar: Color::Rgb(22, 22, 22),
    bg_titlebar: Color::Rgb(26, 26, 26),
    bg_tabbar: Color::Rgb(22, 22, 22),
    bg_tab_active: Color::Rgb(18, 18, 18),
    bg_tab_inactive: Color::Rgb(30, 30, 30),
    bg_statusbar: Color::Rgb(26, 26, 26),
    bg_popup: Color::Rgb(22, 22, 22),
    fg_default: Color::Rgb(224, 224, 224),
    fg_muted: Color::Rgb(110, 110, 110),
    fg_statusbar: Color::Rgb(224, 224, 224),
    fg_line_number: Color::Rgb(74, 74, 74),
    fg_line_number_active: Color::Rgb(208, 208, 208),
    fg_dir: Color::Rgb(201, 168, 118),
    fg_accent: Color::Rgb(160, 160, 160),
    selected_bg: Color::Rgb(42, 42, 42),
};

impl Theme {
    pub fn by_name(name: &str) -> Theme {
        match name.to_lowercase().as_str() {
            "vscode" | "vscode-dark-plus" => VSCODE_DARK_PLUS,
            "zed" | "one-dark" => ZED_ONE_DARK,
            "matte" | "matte-black" => MATTE_BLACK,
            _ => VSCODE_DARK_PLUS, 
        }
    }
}