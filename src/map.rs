use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

#[derive(Clone, Copy)]
pub enum Tile {
    Empty,
    Obstacle,
    Energy,
    Mineral,
    Science,
}

pub struct Map {
    pub grid: Vec<Vec<Tile>>,
    pub cols: usize,
}

impl Map {
    pub fn generate(rows: usize, cols: usize, seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        let mut grid = vec![vec![Tile::Empty; cols]; rows];

        for row in 0..rows {
            for col in 0..cols {
                let roll = rng.random_range(0..100);
                grid[row][col] = match roll {
                    0..=5 => Tile::Obstacle,
                    6..=8 => Tile::Energy,
                    9..=11 => Tile::Mineral,
                    12..=13 => Tile::Science,
                    _ => Tile::Empty,
                };
            }
        }

        Self { grid, cols }
    }
}
