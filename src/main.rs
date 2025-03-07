mod map;

fn main() {
    println!("Hello, world!");
    let map = map::Map::new(100, 100, 0);
    map.display();
}
