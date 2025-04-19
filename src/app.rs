use crate::map::{Map, Tile};
use crate::robot::Robot;
use crate::robot::RobotModule;
use std::collections::HashSet;

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
        let robots = vec![
            Robot::new(
                1,
                (0, 0),
                vec![
                    RobotModule::Explorer,
                    RobotModule::Collector,
                    RobotModule::Scanner,
                    RobotModule::Sensor,
                ],
            ),
            Robot::new(
                2,
                (map.grid.len() - 1, map.cols - 1),
                vec![
                    RobotModule::Explorer,
                    RobotModule::Collector,
                    RobotModule::Scanner,
                    RobotModule::Sensor,
                ],
            ),
        ];
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

        let robot_snapshots: Vec<(usize, (usize, usize))> =
            self.robots.iter().map(|r| (r.id, r.position)).collect();

        for robot in &mut self.robots {
            if robot.modules.contains(&RobotModule::Scanner) {
                robot.scan_surroundings(&self.map);
            }

            if robot.modules.contains(&RobotModule::Explorer) {
                let nearby_robots = robot.scan_for_robots(&robot_snapshots);
                let mut occupied: HashSet<(usize, usize)> =
                    robot_snapshots.iter().map(|(_, pos)| *pos).collect();

                for pos in &nearby_robots {
                    occupied.insert(*pos);
                }

                occupied.remove(&robot.position);
                robot.smart_move(&self.map, &occupied);
            }

            if robot.modules.contains(&RobotModule::Collector) {
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

        self.tick_count > 100
    }
}
