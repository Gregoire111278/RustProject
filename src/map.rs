use noise::{NoiseFn, Perlin};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Tile {
    Empty,
    Obstacle,
    Energy,
    Mineral,
    Science,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapDiff(pub Vec<((usize, usize), Option<Tile>, Tile)>);

#[allow(dead_code)]
impl MapDiff {
    pub fn apply(&self, map: &mut Map) {
        for &((r, c), _before, after) in &self.0 {
            if r < map.grid.len() && c < map.cols {
                map.grid[r][c] = after;
            }
        }
    }

    pub fn merge(&mut self, other: MapDiff) {
        self.0.extend(other.0);
    }

    pub fn apply_to_known_map(&self, known_map: &mut HashMap<(usize, usize), Tile>) {
        for &((r, c), _before, after) in &self.0 {
            known_map.insert((r, c), after);
        }
    }
}

#[derive(Debug, Clone)]
pub struct Map {
    pub grid: Vec<Vec<Tile>>,
    pub cols: usize,
}

impl Map {
    pub fn generate(rows: usize, cols: usize, seed: u64) -> Self {
        let perlin = Perlin::default();
        let mut rng = StdRng::seed_from_u64(seed);
        let mut grid = vec![vec![Tile::Empty; cols]; rows];

        for row in 0..rows {
            for col in 0..cols {
                let x = row as f64 / 10.0;
                let y = col as f64 / 10.0;
                let mut val = perlin.get([x, y, seed as f64]);

                val = (val + 1.0) / 2.0;

                grid[row][col] = if val > 0.7 {
                    Tile::Obstacle
                } else if val > 0.4 {
                    if rng.gen_bool(0.5) {
                        Tile::Mineral
                    } else {
                        Tile::Energy
                    }
                } else if val > 0.2 {
                    if rng.gen_bool(0.3) {
                        Tile::Science
                    } else {
                        Tile::Empty
                    }
                } else {
                    Tile::Empty
                };
            }
        }

        Self { grid, cols }
    }

    pub fn generate_with_dynamic_seed(rows: usize, cols: usize) -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Self::generate(rows, cols, seed)
    }
}
