// main.rs
mod app;
mod cursor;
mod document;
mod mode;
mod ui;

use std::io;
use std::env;

use app::App;

fn main() -> io::Result<()> {
    let filename = env::args().nth(1);
    
    ratatui::run(|terminal| {
        let mut app = App::new(filename);
        app.run(terminal)
    })
}