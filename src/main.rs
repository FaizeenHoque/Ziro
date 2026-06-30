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
    Insert,
    Command,
}

#[derive(Debug, Default)]
pub struct App {
    document: Document,
    cursor: Cursor,
    exit: bool,
    mode: Mode,
    show_dialogue: bool,
    command_input: String,
}

#[derive(Debug)]
struct Document {
    lines: Vec<String>
}

impl Default for Document {
    fn default() -> Self {
        Self {
            lines: vec![String::new()],
        }
    }
}

#[derive(Debug, Default)]
struct Cursor {
    x: usize,
    y: usize
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

        frame.set_cursor_position((
                self.cursor.x as u16,
                self.cursor.y as u16,
            ));
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
            // Document Typing
            KeyCode::Char(c) if self.mode == Mode::Insert => {
                self.insert_char(c);
            }
            KeyCode::Enter if self.mode == Mode::Insert => {
                let new_line = self.document.lines[self.cursor.y]
                    .split_off(self.cursor.x);

                self.document.lines.insert(
                    self.cursor.y + 1,
                    new_line,
                );

                self.cursor.y += 1;
                self.cursor.x = 0;
            }
            KeyCode::Backspace if self.mode == Mode::Insert => {
                self.backspace();
            }

            // Document Movement Keybinds
            KeyCode::Up if self.mode == Mode::Insert => {
                self.cursor.y = self.cursor.y.saturating_sub(1);
            }
            KeyCode::Down if self.mode == Mode::Insert => {
                if self.cursor.y + 1 < self.document.lines.len() {
                    self.cursor.y += 1;

                    self.cursor.x = self.cursor.x
                        .min(self.document.lines[self.cursor.y].len());
                }
            }
            KeyCode::Left if self.mode == Mode::Insert => {
                self.cursor.x = self.cursor.x.saturating_sub(1);
            }
            KeyCode::Right if self.mode == Mode::Insert => {
                let line_length = self.document.lines[self.cursor.y].len();

                if self.cursor.x < line_length {
                    self.cursor.x += 1;
                }
            }

            // Command Dialogue keybinds
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
                self.mode = Mode::Insert;
            }
            KeyCode::Esc => {
                if self.show_dialogue == false {
                    self.show_dialogue = true;
                    self.mode = Mode::Command;
                } else {
                    self.command_input.clear();
                    self.show_dialogue = false;
                    self.mode = Mode::Insert;
                }
            }
            
            _ => {}
        }
    }

    fn insert_char(&mut self, c: char) {
        if self.cursor.y >= self.document.lines.len() {
            self.document.lines.push(String::new());
        }

        self.document.lines[self.cursor.y]
            .insert(self.cursor.x, c);

        self.cursor.x += 1;
    }

    fn backspace(&mut self) {
        if self.cursor.x > 0 {
            self.document.lines[self.cursor.y]
                .remove(self.cursor.x - 1);

            self.cursor.x -= 1;
        } 
        else if self.cursor.y > 0 {
            let current_line = self.document.lines.remove(self.cursor.y);

            self.cursor.y -= 1;

            let previous_length = self.document.lines[self.cursor.y].len();

            self.document.lines[self.cursor.y].push_str(&current_line);

            self.cursor.x = previous_length;
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::vertical([
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(area);

        let text: Vec<Line> = self.document.lines
            .iter()
            .map(|line| Line::from(line.as_str()))
            .collect();

        Paragraph::new(text)
            .render(chunks[0], buf);

        let status = match self.mode {
            Mode::Insert => "INSERT",
            Mode::Command => &format!("> {}", self.command_input),
        };

        Paragraph::new(status)
            .render(chunks[1], buf);
    }
}

fn main() -> io::Result<()> {
    ratatui::run(|terminal| {
        App::default().run(terminal)
    })
}