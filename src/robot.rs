#[derive(Debug)]
pub enum RobotModule {
    Explorer,
    Collector,
    // Scientist,
    // Battery,
    // Drill,
    // CommModule,
    // Builder,
}

#[derive(Debug)]
pub struct Robot {
    pub id: usize,
    pub position: (usize, usize),
    pub modules: Vec<RobotModule>,
    pub energy_collected: u32,
    pub mineral_collected: u32,
}

impl Robot {
    pub fn new(id: usize, position: (usize, usize), modules: Vec<RobotModule>) -> Self {
        Self {
            id,
            position,
            modules,
            energy_collected: 0,
            mineral_collected: 0,
        }
    }

    pub fn move_right(&mut self, map_rows: usize, map_cols: usize) {
        let (row, col) = self.position;
        if row < map_rows && col + 1 < map_cols {
            self.position = (row, col + 1);
        }
    }
}
