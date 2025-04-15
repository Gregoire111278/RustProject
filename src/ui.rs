use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, List, ListItem, Paragraph, Row, Table},
    Frame,
};

use crate::app::App;
use crate::map::Tile;
use std::thread;
use std::time::Duration;

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
                                " 🤖 ",
                                Style::default()
                                    .fg(Color::Cyan)
                                    .add_modifier(Modifier::BOLD),
                            )
                        } else {
                            match tile {
                                Tile::Empty => (" · ", Style::default().fg(Color::DarkGray)),
                                Tile::Obstacle => (" # ", Style::default().fg(Color::Red)),
                                Tile::Energy => (" E ", Style::default().fg(Color::Yellow)),
                                Tile::Mineral => (" M ", Style::default().fg(Color::Blue)),
                                Tile::Science => (" S ", Style::default().fg(Color::Green)),
                            }
                        };

                    Cell::from(Span::styled(symbol, style))
                })
                .collect();
            Row::new(cells)
        })
        .collect();

    let map_widget = Table::default()
        .block(Block::default().title("PlanetMap").borders(Borders::ALL))
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

    let robot_panel = List::new(robot_info)
        .block(Block::default().title("Robots Info").borders(Borders::ALL))
        .style(Style::default().fg(Color::Cyan));

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(3)])
        .split(chunks[1]);

    f.render_widget(robot_panel, right_chunks[0]);

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

    thread::sleep(Duration::from_millis(150));

    let legend_lines = vec![
        Line::from(" 🤖  - Robot"),
        Line::from(" #  - Obstacle"),
        Line::from(" E  - Energy"),
        Line::from(" M  - Mineral"),
        Line::from(" S  - Science"),
        Line::from(" ·  - Empty"),
    ];
    let legend =
        Paragraph::new(legend_lines).block(Block::default().title("Legend").borders(Borders::ALL));
    f.render_widget(legend, right_chunks[1]);
}
