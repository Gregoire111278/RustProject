use noise::{NoiseFn, Perlin};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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



// ##################################### UNIT TESTS #############################################
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_map_dimensions() {
        let rows = 10;
        let cols = 15;
        let seed = 42;
        let map = Map::generate(rows, cols, seed);

        assert_eq!(map.grid.len(), rows);
        for row in &map.grid {
            assert_eq!(row.len(), cols);
        }
        assert_eq!(map.cols, cols);
    }

    #[test]
    fn test_map_tile_variants() {
        let map = Map::generate(10, 10, 123);
        let valid_tiles: HashSet<Tile> = [
            Tile::Empty,
            Tile::Obstacle,
            Tile::Energy,
            Tile::Mineral,
            Tile::Science,
        ]
        .into_iter()
        .collect();

        for row in &map.grid {
            for tile in row {
                assert!(valid_tiles.contains(tile), "Invalid tile: {:?}", tile);
            }
        }
    }

    #[test]
    fn test_map_seed_repetability() {
        let map1 = Map::generate(10, 10, 999);
        let map2 = Map::generate(10, 10, 999);

        assert_eq!(map1.grid, map2.grid);
    }

    #[test]
    fn test_generate_with_dynamic_seed_dimensions() {
        let map = Map::generate_with_dynamic_seed(5, 5);
        assert_eq!(map.grid.len(), 5);
        for row in &map.grid {
            assert_eq!(row.len(), 5);
        }
    }
}
