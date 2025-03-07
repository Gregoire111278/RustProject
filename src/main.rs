use crate::map::Map;

mod map;

fn main() {
    println!("Hello, world!");
    let map = Map::new(20, 10, 50);
    map.display();
}
