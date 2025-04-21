use RustProject::map::Tile;
use RustProject::map::Map;



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
