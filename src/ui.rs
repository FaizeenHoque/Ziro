use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{
        Constraint,
        Layout,
    },
    text::Line,
    widgets::{
        Paragraph,
        Widget,
    },
};

use crate::app::App;


pub fn draw(
    frame: &mut Frame,
    app: &App,
) {
    frame.render_widget(
        app,
        frame.area(),
    );
}


impl Widget for &App {

    fn render(
        self,
        area: ratatui::layout::Rect,
        buf: &mut Buffer,
    ) {

        let chunks = Layout::vertical([
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(area);


        let text: Vec<Line> =
            self.document.lines
            .iter()
            .map(|line| Line::from(line.as_str()))
            .collect();


        Paragraph::new(text)
            .render(chunks[0], buf);


        let status = match self.mode {
            crate::mode::Mode::Insert => "INSERT",
            crate::mode::Mode::Command =>
                &self.command_input,
        };


        Paragraph::new(status)
            .render(chunks[1], buf);
    }
}