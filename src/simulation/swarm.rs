use super::robot::Robot;
use rayon::prelude::*;

pub struct Swarm {
    robots: Vec<Robot>,
}

impl Swarm {
    pub fn new() -> Self {
        let mut swarm = Swarm { robots: Vec::new() };
        swarm.robots.push(Robot::new(1, 0, 0));
        swarm
    }

    pub fn move_robots(&mut self) {
        self.robots.par_iter_mut().for_each(|robot| {
            robot.move_robot(1, 1);
            robot.status();
        });
    }
}
