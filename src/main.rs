mod app;
mod cursor;
mod document;
mod mode;
mod ui;

use std::io;

use app::App;

fn main() -> io::Result<()> {
    ratatui::run(|terminal| {
        App::default().run(terminal)
    })
}