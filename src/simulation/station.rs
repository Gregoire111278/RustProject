use super::robot::Robot;
use std::collections::HashMap;

pub struct Station {
    data: HashMap<u32, String>,
}

impl Station {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn collect_data(&mut self, robot: &Robot) {
        self.data
            .insert(robot.id, format!("Robot at {:?}", robot.position));
    }
}
