#[derive(Debug)]
pub enum RobotModule {
    Explorer,
    Collector,
    Scanner,
}

#[derive(Debug)]
pub struct Robot {
    pub known_map: std::collections::HashMap<(usize, usize), Tile>,
    pub id: usize,
    pub position: (usize, usize),
    pub modules: Vec<RobotModule>,
    pub energy_collected: u32,
    pub mineral_collected: u32,
}

use crate::map::Tile;

impl Robot {
    pub fn new(id: usize, position: (usize, usize), modules: Vec<RobotModule>) -> Self {
        Self {
            known_map: std::collections::HashMap::new(),
            id,
            position,
            modules,
            energy_collected: 0,
            mineral_collected: 0,
        }
    }

    pub fn scan_surroundings(&mut self, map: &crate::map::Map) {
        let (row, col) = self.position;
        for dr in -1..=1 {
            for dc in -1..=1 {
                let r = row.wrapping_add(dr as usize);
                let c = col.wrapping_add(dc as usize);
                if r < map.grid.len() && c < map.cols {
                    let tile = map.grid[r][c];
                    self.known_map.insert((r, c), tile);
                }
            }
        }
    }

    pub fn smart_move(&mut self, map: &crate::map::Map) {
        let directions = [
            (0, 1),          // right
            (1, 0),          // down
            (0, usize::MAX), // left
            (usize::MAX, 0), // up
        ];

        let mut preferred_move = None;
        let mut fallback_move = None;

        for &(dr, dc) in &directions {
            let new_row = self.position.0.wrapping_add(dr);
            let new_col = self.position.1.wrapping_add(dc);

            if new_row < map.grid.len() && new_col < map.cols {
                match map.grid[new_row][new_col] {
                    Tile::Obstacle => continue,
                    Tile::Energy | Tile::Mineral => {
                        preferred_move = Some((new_row, new_col));
                        break;
                    }
                    Tile::Empty | Tile::Science => {
                        if fallback_move.is_none() {
                            fallback_move = Some((new_row, new_col));
                        }
                    }
                }
            }
        }

        if let Some(target) = preferred_move.or(fallback_move) {
            self.position = target;
        }
    }
}
