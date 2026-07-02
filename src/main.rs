// main.rs
mod app;
mod cursor;
mod document;
mod ui;
mod syntax;

use std::io;
use std::env;
use std::io::stdout;

use app::App;
use crossterm::event::EnableMouseCapture;
use crossterm::execute;

fn main() -> io::Result<()> {
    execute!(stdout(), EnableMouseCapture)?;

    let filename = env::args().nth(1);
    
    ratatui::run(|terminal| {
        App::new(filename)?.run(terminal)
    })?;

    execute!(stdout(), crossterm::event::DisableMouseCapture)?;
    Ok(())
}