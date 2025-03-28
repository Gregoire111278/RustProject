mod simulation;
use simulation::map::Map;

fn main() {
    println!("starting ...");
    let map = Map::new(20, 42);
    map.display();
}
