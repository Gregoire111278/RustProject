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
        let mut rng = rand::thread_rng();

    }
}