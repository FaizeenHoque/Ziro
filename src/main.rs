// main.rs
mod app;
mod cursor;
mod document;
mod mode;
mod ui;
mod syntax;

use std::io;
use std::env;

use app::App;

fn main() -> io::Result<()> {
    let filename = env::args().nth(1);
    
    ratatui::run(|terminal| {
        App::new(filename)?.run(terminal)
    })
}