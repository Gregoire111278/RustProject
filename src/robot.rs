#[derive(Debug)]
pub struct Robot {
    pub id: usize,
    pub position: (usize, usize),
}

impl Robot {
    pub fn new(id: usize, position: (usize, usize)) -> Self {
        Self { id, position }
    }

    pub fn move_right(&mut self, map_rows: usize, map_cols: usize) {
        let (row, col) = self.position;

        if row < map_rows && col + 1 < map_cols {
            self.position = (row, col + 1);
        }
    }
}
