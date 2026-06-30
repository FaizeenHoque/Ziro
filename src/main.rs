use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};

use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Flex, Layout, Rect},
    style::{Color, Stylize},
    text::{Line, Text},
    widgets::{Block, BorderType, Clear, Paragraph, Widget},
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
    show_dialogue: bool,
    command_input: String
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| {
                self.draw(frame);
            })?;

            self.handle_events()?;
        }

        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());

        if self.show_dialogue {
            render_command_dialogue(frame, self);
        }
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
            KeyCode::Esc => {
                if self.show_dialogue == false {
                    self.show_dialogue = true;
                    self.mode = Mode::Command;
                } else {
                    self.show_dialogue = false;
                    self.mode = Mode::Normal;
                }
            }

            KeyCode::Char(c) if self.mode == Mode::Command => {
                self.command_input.push(c);
            }

            KeyCode::Backspace if self.mode == Mode::Command => {
                self.command_input.pop();
            }

            KeyCode::Enter if self.mode == Mode::Command => {
                if self.command_input == ":q" {
                    self.exit();
                }
                self.command_input.clear();
                self.show_dialogue = false;
                self.mode = Mode::Normal;
            }

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

fn render_command_dialogue(frame: &mut Frame, app: &App) {
    let area = frame.area();

    frame.render_widget(Clear, area);

    let popup_area = Layout::vertical([
        Constraint::Percentage(30),
        Constraint::Percentage(15),
        Constraint::Percentage(30),
    ])
    .split(area)[1];

    let popup_area_horiz = Layout::horizontal([
        Constraint::Percentage(30),
        Constraint::Percentage(40),
        Constraint::Percentage(30),
    ])
    .split(popup_area)[1];

    // 4. Render the dialogue widget
    let dialog_block = Block::bordered()
        .title("Command")
        .border_type(BorderType::Rounded)
        .bg(Color::Black);

    let paragraph = Paragraph::new(format!("> {}", app.command_input))
        .block(dialog_block)
        .alignment(Alignment::Left);

    frame.render_widget(Clear, popup_area_horiz);
    frame.render_widget(paragraph, popup_area_horiz);
}

fn main() -> io::Result<()> {
    ratatui::run(|terminal| {
        App::default().run(terminal)
    })
}