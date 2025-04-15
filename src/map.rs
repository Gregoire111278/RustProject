use noise::{NoiseFn, Perlin};

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
        let perlin = Perlin::default();
        let mut grid = vec![vec![Tile::Empty; cols]; rows];

        for row in 0..rows {
            for col in 0..cols {
                let x = row as f64 / 10.0;
                let y = col as f64 / 10.0;
                let val = perlin.get([x, y, seed as f64]);

                grid[row][col] = if val > 0.6 {
                    Tile::Obstacle
                } else if val > 0.4 {
                    Tile::Mineral
                } else if val > 0.2 {
                    Tile::Energy
                } else if val > 0.1 {
                    Tile::Science
                } else {
                    Tile::Empty
                };
            }
        }

        Self { grid, cols }
    }
}
