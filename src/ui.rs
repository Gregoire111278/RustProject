use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, List, ListItem, Paragraph, Row, Table},
    Frame,
};

use crate::app::App;
use crate::map::Tile;

pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(f.size());

    let map_grid: Vec<Row> = app
        .map
        .grid
        .iter()
        .enumerate()
        .map(|(row_idx, row)| {
            let cells: Vec<Cell> = row
                .iter()
                .enumerate()
                .map(|(col_idx, tile)| {
                    let (symbol, style) =
                        if app.robots.iter().any(|r| r.position == (row_idx, col_idx)) {
                            (
                                "🤖 ",
                                Style::default()
                                    .fg(Color::Cyan)
                                    .add_modifier(Modifier::BOLD),
                            )
                        } else {
                            match tile {
                                Tile::Empty => (" . ", Style::default().fg(Color::DarkGray)),
                                Tile::Obstacle => (" # ", Style::default().fg(Color::Red)),
                                Tile::Energy => ("⚡ ", Style::default().fg(Color::Yellow)),
                                Tile::Mineral => ("⛏ ", Style::default().fg(Color::Blue)),
                                Tile::Science => ("🔬 ", Style::default().fg(Color::Green)),
                            }
                        };

                    Cell::from(Span::styled(symbol, style))
                })
                .collect();
            Row::new(cells)
        })
        .collect();

    let map_widget = Table::default()
        .block(Block::default().title("Planet Map").borders(Borders::ALL))
        .widths(&vec![Constraint::Length(3); app.map.cols])
        .rows(map_grid);

    f.render_widget(map_widget, chunks[0]);

    let robot_info: Vec<ListItem> = app
        .robots
        .iter()
        .map(|r| {
            ListItem::new(Line::from(vec![Span::styled(
                format!("Robot #{} at ({}, {})", r.id, r.position.0, r.position.1),
                Style::default().fg(Color::Cyan),
            )]))
        })
        .collect();

    let robot_panel =
        List::new(robot_info).block(Block::default().title("Robots Info").borders(Borders::ALL));

    f.render_widget(robot_panel, chunks[1]);

    let status_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .split(f.size());

    let status = Paragraph::new(Line::from(vec![Span::styled(
        format!("Tick: {} | Robots: {}", app.tick_count, app.robots.len()),
        Style::default().fg(Color::White),
    )]))
    .block(Block::default().title("Status").borders(Borders::ALL));

    f.render_widget(status, status_chunks[1]);
}
