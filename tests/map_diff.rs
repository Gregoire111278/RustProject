use rust_project::map::{Map, MapDiff, Tile};

#[test]
fn diff_apply() {
    let mut map = Map::generate(3, 3, 42);
    map.grid[1][1] = Tile::Energy;

    let diff = MapDiff(vec![((1, 1), Some(Tile::Energy), Tile::Empty)]);
    diff.apply(&mut map);

    assert_eq!(map.grid[1][1], Tile::Empty);
}
