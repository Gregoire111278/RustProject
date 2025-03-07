use noise::{NoiseFn, Perlin};
use rand::Rng;


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
    pub fn new(width: usize, height: usize, seed: usize) -> Self {
        let perlin = Perlin::new();
        let mut rng = rand::thread_rng();
        let data = (0..height)
            .map(|y| {
                (0..width)
                    .map(|x| {
                        let value = perlin.get([x as f64 / 10.0, y as f64 / 10.0, seed as f64]);
                        if value > 0.7 {
                            Tile::Obstacle
                        } else if value > 0.5 {
                            Tile::Energy
                        } else if value > 0.3 {
                            Tile::Mineral
                        } else if value > 0.1 {
                            Tile::Science
                        } else {
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
    pub fn get_tile(&self, x: usize, y: usize) -> Tile {
            self.data[y][x]
        }

        pub fn set_tile(&mut self, x: usize, y: usize, tile: Tile) {
            self.data[y][x] = tile;
        }
    }

