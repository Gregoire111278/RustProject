use crate::map::{Map, Tile};
use crate::robot::Robot;
use crate::robot::RobotModule;

pub struct App {
    pub map: Map,
    pub robots: Vec<Robot>,
    pub tick_count: u64,
    pub collected_energy: u32,
    pub collected_mineral: u32,
}

impl App {
    pub fn new() -> Self {
        let map = Map::generate_with_dynamic_seed(25, 26);
        let robots = vec![Robot::new(
            1,
            (0, 0),
            vec![
                RobotModule::Explorer,
                RobotModule::Collector,
                RobotModule::Scanner,
            ],
        )];
        Self {
            map,
            robots,
            tick_count: 0,
            collected_energy: 0,
            collected_mineral: 0,
        }
    }

    pub fn tick(&mut self) -> bool {
        self.tick_count += 1;

        for robot in &mut self.robots {
            if robot
                .modules
                .iter()
                .any(|m| matches!(m, RobotModule::Scanner))
            {
                robot.scan_surroundings(&self.map);
            }

            if robot
                .modules
                .iter()
                .any(|m| matches!(m, RobotModule::Explorer))
            {
                robot.move_right(self.map.grid.len(), self.map.cols);
            }

            if robot
                .modules
                .iter()
                .any(|m| matches!(m, RobotModule::Collector))
            {
                let (row, col) = robot.position;
                if row < self.map.grid.len() && col < self.map.cols {
                    let tile = &mut self.map.grid[row][col];
                    match tile {
                        Tile::Energy => {
                            robot.energy_collected += 1;
                            self.collected_energy += 1;
                            *tile = Tile::Empty;
                        }
                        Tile::Mineral => {
                            robot.mineral_collected += 1;
                            self.collected_mineral += 1;
                            *tile = Tile::Empty;
                        }
                        _ => {}
                    }
                }
            }
        }

        self.tick_count > 40
    }
}
