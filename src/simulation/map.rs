use rand::SeedableRng;
use rand::rngs::StdRng;
use noise::{NoiseFn, Perlin};
use rand::Rng;

pub struct Map {
    size: usize,
    grid: Vec<Vec<char>>,
}

impl Map {
    pub fn new(size: usize, seed: u32) -> Self {
        let perlin = Perlin::new(seed);
        let mut rng = StdRng::seed_from_u64(seed as u64);

        let mut grid = vec![vec!['⬜'; size]; size];

        for y in 0..size {
            for x in 0..size {
                let noise_value = perlin.get([x as f64 / 10.0, y as f64 / 10.0]);

                grid[y][x] = if noise_value > 0.2 {
                    '⛰'
                } else if rng.gen_range(0..100) < 10 {
                    '💎'
                } else if rng.gen_range(0..100) < 5 {
                    '⚡'
                } else if rng.gen_range(0..100) < 3 {
                    '🔬'
                } else {
                    '⬜'
                };
            }
        }

        Self { size, grid }
    }

    pub fn display(&self) {
        println!("🗺️  Generated Map:");
        for row in &self.grid {
            for &cell in row {
                print!("{}", cell);
            }
            println!();
        }
        println!("-------------------");
    }
}
