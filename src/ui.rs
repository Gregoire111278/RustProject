use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Scrollbar, ScrollbarState, Table},
    Frame,
};

use crate::app::App;
use crate::map::Tile;
use std::thread;
use std::time::Duration;

fn make_state(content_len: usize, view_height: u16, offset: usize) -> ScrollbarState {
    ScrollbarState::new(content_len)
        .position(offset.min(content_len.saturating_sub(view_height as usize)))
}

pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
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
                        } else if app
                            .robots
                            .iter()
                            .any(|r| r.known_map.contains_key(&(row_idx, col_idx)))
                        {
                            match tile {
                                Tile::Empty => (
                                    " · ",
                                    Style::default()
                                        .fg(Color::DarkGray)
                                        .add_modifier(Modifier::DIM),
                                ),
                                Tile::Obstacle => (
                                    " # ",
                                    Style::default().fg(Color::Red).add_modifier(Modifier::DIM),
                                ),
                                Tile::Energy => (
                                    " E ",
                                    Style::default()
                                        .fg(Color::Yellow)
                                        .add_modifier(Modifier::DIM),
                                ),
                                Tile::Mineral => (
                                    " M ",
                                    Style::default().fg(Color::Blue).add_modifier(Modifier::DIM),
                                ),
                                Tile::Science => (
                                    " S ",
                                    Style::default()
                                        .fg(Color::Green)
                                        .add_modifier(Modifier::DIM),
                                ),
                            }
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

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Percentage(30),
            Constraint::Percentage(30),
        ])
        .split(chunks[1]);

    let robot_info_text = app
        .robots
        .iter()
        .map(|r| {
            let modules = r
                .modules
                .iter()
                .map(|m| format!("{:?}", m))
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "Robot #{} at ({}, {})\n  Modules: [{}]\n  Energy: {}  Mineral: {}  Known tiles: {}\n  Nearby Robots: {}\n",
                r.id,
                r.position.0,
                r.position.1,
                modules,
                r.energy_collected,
                r.mineral_collected,
                r.known_map.len(),
                app.robots.iter()
                    .filter(|other| {
                        other.id != r.id
                            && (r.position.0 as isize - other.position.0 as isize).abs() <= 1
                            && (r.position.1 as isize - other.position.1 as isize).abs() <= 1
                    })
                    .count()
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let mut robots_state = make_state(
        app.robots.len(),
        right_chunks[0].height,
        app.robots_scroll as usize,
    );

    let robot_panel = Paragraph::new(robot_info_text)
        .block(Block::default().title("Robots Info").borders(Borders::ALL))
        .style(Style::default().fg(Color::Cyan))
        .wrap(ratatui::widgets::Wrap { trim: false })
        .scroll((app.robots_scroll, 0));

    f.render_widget(robot_panel, right_chunks[0]);

    let robots_bar = Scrollbar::default()
        .orientation(ratatui::widgets::ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"))
        .style(Style::default().fg(Color::Cyan));

    f.render_stateful_widget(robots_bar, right_chunks[0], &mut robots_state);

    let log_lines: Vec<Line> = app
        .logs
        .iter()
        .rev()
        .take(15)
        .map(|l| Line::from(l.clone()))
        .collect();

    let mut logs_state = make_state(
        app.logs.len(),
        right_chunks[1].height,
        app.logs_scroll as usize,
    );

    let logs_widget = Paragraph::new(log_lines)
        .block(Block::default().title("Station Logs").borders(Borders::ALL))
        .wrap(ratatui::widgets::Wrap { trim: false })
        .scroll((app.logs_scroll, 0));

    f.render_widget(logs_widget, right_chunks[1]);

    let logs_bar = Scrollbar::default()
        .orientation(ratatui::widgets::ScrollbarOrientation::VerticalRight)
        .style(Style::default().fg(Color::Yellow));

    f.render_stateful_widget(logs_bar, right_chunks[1], &mut logs_state);

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
    f.render_widget(legend, right_chunks[2]);

    let status_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .split(f.size());

    let status = Paragraph::new(Line::from(vec![Span::styled(
        format!(
            "Tick: {} | Robots: {} | Energy: {} | Mineral: {} | Master ver: {}",
            app.tick_count,
            app.robots.len(),
            app.collected_energy,
            app.collected_mineral,
            app.master_version,
        ),
        Style::default().fg(Color::White),
    )]))
    .block(Block::default().title("Status").borders(Borders::ALL));

    f.render_widget(status, status_chunks[1]);

    thread::sleep(Duration::from_millis(150));
}
