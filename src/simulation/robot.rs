#[derive(Debug)]
pub struct Robot {
    pub id: u32,
    battery: f32,
    pub position: (usize, usize),
}

impl Robot {
    pub fn new(id: u32, x: usize, y: usize) -> Self {
        Self {
            id,
            battery: 100.0,
            position: (x, y),
        }
    }

    pub fn move_robot(&mut self, new_x: usize, new_y: usize) {
        self.position = (new_x, new_y);
        self.battery -= 5.0;
    }

    pub fn status(&self) {
        println!(
            "🤖 Robot {} at {:?}, Battery: {:.1}%",
            self.id, self.position, self.battery
        );
    }
}
