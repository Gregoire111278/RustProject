use noise::{NoiseFn, Perlin};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

pub struct Map {
    width: usize,
    height: usize,
    data: Vec<Vec<Tile>>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Tile {
    Empty,
    Obstacle,
    Energy,
    Mineral,
    Science,
}

impl Map {
    pub fn new(width: usize, height: usize, seed: u64) -> Self {
        let perlin = Perlin::new();
        let mut rng = StdRng::seed_from_u64(seed);

        let data = (0..height)
            .map(|y| {
                (0..width)
                    .map(|x| {
                        let noise_value = perlin.get([x as f64 / 10.0, y as f64 / 10.0, seed as f64]);
                        let random_adjustment: f64 = rng.random_range(-0.99..0.99);

                        let value = noise_value + random_adjustment;
                        if value > 0.9 {
                            Tile::Obstacle
                        } else if value > 0.8 {
                            Tile::Energy
                        } else if value > 0.7 {
                            Tile::Mineral
                        } else if value > 0.59 {
                            Tile::Science
                        } else if value > -0.5 {
                            Tile::Empty
                        }
                        else {
                            Tile::Empty
                        }
                    })
                    .collect()
            })
            .collect();

        Map {
            width,
            height,
            data,
        }
    }

    pub fn display(&self) {
        for row in &self.data {
            for &tile in row {
                let symbol = match tile {
                    Tile::Empty => ".",
                    Tile::Obstacle => "🚧",
                    Tile::Energy => "E",
                    Tile::Mineral => "M",
                    Tile::Science => "S",
                };
                print!("{}", symbol);
            }
            println!();
        }
    }
    pub fn get_tile(&self, x: usize, y: usize) -> Option<Tile> {
        if x < self.width && y < self.height {
            Some(self.data[y][x])
        } else {
            None
        }
    }

    pub fn set_tile(&mut self, x: usize, y: usize, tile: Tile) {
        if x < self.width && y < self.height {
            self.data[y][x] = tile;
        }
    }
}

