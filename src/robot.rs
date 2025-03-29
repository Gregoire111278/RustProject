#[derive(Debug)]
pub struct Robot {
    pub id: usize,
    pub position: (usize, usize),
}

impl Robot {
    pub fn new(id: usize, position: (usize, usize)) -> Self {
        Self { id, position }
    }
}
