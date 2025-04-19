mod app;
mod map;
mod robot;
mod station;
mod ui;
mod utils;

use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::sync::mpsc;
use std::thread;
use std::{error::Error, io};

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;

    let (tx_report, rx_report) = mpsc::channel();
    let (tx_cmd, rx_cmd) = mpsc::channel();

    thread::spawn(move || {
        let station = station::Station::new(rx_report, tx_cmd);
        station.run();
    });

    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = app::App::new(tx_report.clone(), rx_cmd);

    loop {
        terminal.draw(|f| ui::render(f, &app))?;
        if app.tick() {
            break;
        }
    }

    disable_raw_mode()?;
    Ok(())
}
