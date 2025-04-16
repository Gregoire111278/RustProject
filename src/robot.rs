#[derive(Debug)]
pub enum RobotModule {
    Explorer,
    Collector,
    Scanner,
    // Scientist,
    // Battery,
    // Drill,
    // CommModule,
    // Builder,
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

    pub fn move_right(&mut self, map_rows: usize, map_cols: usize) {
        let (row, col) = self.position;
        if row < map_rows && col + 1 < map_cols {
            self.position = (row, col + 1);
        }
    }
}
