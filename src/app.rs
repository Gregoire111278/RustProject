use crate::map::Map;
use crate::robot::Robot;
use crate::robot::RobotModule;

pub struct App {
    pub map: Map,
    pub robots: Vec<Robot>,
    pub tick_count: u64,
}

impl App {
    pub fn new() -> Self {
        let map = Map::generate_with_dynamic_seed(25, 26);
        let robots = vec![
            Robot::new(1, (0, 0), vec![RobotModule::Explorer, RobotModule::Collector]),
        ];
        Self {
            map,
            robots,
            tick_count: 0,
        }
    }

    pub fn tick(&mut self) -> bool {
        self.tick_count += 1;

        for robot in &mut self.robots {
            robot.move_right(self.map.grid.len(), self.map.cols);
        }

        self.tick_count > 40
    }
}
