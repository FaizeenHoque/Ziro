use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame, buffer::Buffer, layout::{Alignment, Constraint, Direction, Flex, HorizontalAlignment::Center, Layout, Rect}, style::Stylize, symbols::border, text::{Line, Text}, widgets::{Block, Paragraph, Widget},
};

#[derive(Debug, Default, PartialEq)]
enum Mode {
    #[default]
    Normal,
    Insert,
    Command,
}

#[derive(Debug, Default)]
pub struct App {
    exit: bool,
    mode: Mode,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| {
                return self.draw(frame);
            })?;
            self.handle_events()?;
        }

        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Esc => self.mode = Mode::Command,
            KeyCode::Char('q') if self.mode == Mode::Command => self.exit(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3)])
            .flex(Flex::Center)
            .split(area)[0];

        let area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(40)])
            .flex(Flex::Center)
            .split(area)[0];

        Paragraph::new(Text::from(vec![
            Line::from("Ziro Editor"),
            Line::from("Your editor shouldn't slow you down."),
            Line::from("Instruction Mode <ESC>  |  Exit <:q>"),
        ]))
        .alignment(Alignment::Center)
        .render(area, buf);
    }
}

fn main() -> io::Result<()> {
    ratatui::run(|terminal| {
        App::default().run(terminal)
    })
}