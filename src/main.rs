mod app;
mod map;
mod robot;
mod ui;
mod utils;

use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::{error::Error, io};

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    let backend = CrosstermBackend::new(&mut stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = app::App::new();

    loop {
        terminal.draw(|f| {
            ui::render(f, &app);
        })?;

        if app.tick() {
            break;
        }
    }

    disable_raw_mode()?;
    Ok(())
}
