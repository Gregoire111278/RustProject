mod app;
mod map;
mod robot;
mod station;
mod ui;
mod utils;

use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
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
        if event::poll(Duration::from_millis(20))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Up => app.robots_scroll = app.robots_scroll.saturating_sub(1),
                    KeyCode::Down => app.robots_scroll = app.robots_scroll.saturating_add(1),
                    KeyCode::PageUp => app.logs_scroll = app.logs_scroll.saturating_sub(3),
                    KeyCode::PageDown => app.logs_scroll = app.logs_scroll.saturating_add(3),
                    _ => {}
                }
            }
        }

        let done = app.tick();
        if done {
            break;
        }

        terminal.draw(|f| ui::render(f, &app))?;
    }

    disable_raw_mode()?;
    Ok(())
}
