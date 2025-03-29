use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style}
    ,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;
use crate::map::Tile;

pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Min(5), Constraint::Length(3)])
        .split(f.size());

    let mut map_string = String::new();

    for (row_idx, row) in app.map.grid.iter().enumerate() {
        for (col_idx, tile) in row.iter().enumerate() {
            let robot_here = app.robots.iter().any(|r| r.position == (row_idx, col_idx));
            if robot_here {
                map_string.push_str("🤖");
            } else {
                let symbol = match tile {
                    Tile::Empty => "⬜",
                    Tile::Obstacle => "🪨",
                    Tile::Energy => "🔋",
                    Tile::Mineral => "⛏️",
                    Tile::Science => "🧪",
                };
                map_string.push_str(symbol);
            }
        }
        map_string.push('\n');
    }

    let map_widget = Paragraph::new(map_string)
        .block(Block::default().title("Planet Map").borders(Borders::ALL))
        .style(Style::default().fg(Color::White));

    f.render_widget(map_widget, chunks[0]);

    let info = Paragraph::new(format!(
        "Tick: {} | Robots: {}",
        app.tick_count,
        app.robots.len()
    ))
    .block(Block::default().title("Status").borders(Borders::ALL));

    f.render_widget(info, chunks[1]);
}
