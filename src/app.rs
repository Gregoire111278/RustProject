use crate::map::{Map, Tile};
use crate::robot::Robot;
use crate::robot::{RobotModule, RobotState, PAYLOAD_LIMIT};
use crate::station;
use crate::station::StationCmd;
use std::collections::{HashSet, VecDeque};

const BASE_POS: (usize, usize) = (0, 0);

pub struct App {
    pub map: Map,
    pub robots: Vec<Robot>,
    pub tick_count: u64,
    pub collected_energy: u32,
    pub collected_mineral: u32,

    tx_report: std::sync::mpsc::Sender<station::RobotReport>,
    rx_cmd: std::sync::mpsc::Receiver<StationCmd>,

    pub logs: VecDeque<String>,
}

impl App {
    pub fn new(
        tx_report: std::sync::mpsc::Sender<station::RobotReport>,
        rx_cmd: std::sync::mpsc::Receiver<StationCmd>,
    ) -> Self {
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
            tx_report,
            rx_cmd,
            logs: VecDeque::new(),
        }
    }

    pub fn tick(&mut self) -> bool {
        self.tick_count += 1;

        let snapshots: Vec<(usize, (usize, usize))> =
            self.robots.iter().map(|r| (r.id, r.position)).collect();

        for robot in &mut self.robots {
            if robot.modules.contains(&RobotModule::Scanner) {
                robot.scan_surroundings(&self.map);
            }

            if robot.modules.contains(&RobotModule::Collector) {
                let (row, col) = robot.position;
                if row < self.map.grid.len() && col < self.map.cols {
                    let tile = &mut self.map.grid[row][col];
                    match *tile {
                        Tile::Energy => {
                            robot.energy_collected += 1;
                            self.collected_energy += 1;
                            *tile = Tile::Empty;
                            robot.dirty_tiles.push(((row, col), Tile::Empty));
                        }
                        Tile::Mineral => {
                            robot.mineral_collected += 1;
                            self.collected_mineral += 1;
                            *tile = Tile::Empty;
                            robot.dirty_tiles.push(((row, col), Tile::Empty));
                        }
                        _ => {}
                    }
                }
            }

            if robot.state == RobotState::Exploring
                && robot.energy_collected + robot.mineral_collected >= PAYLOAD_LIMIT
            {
                robot.state = RobotState::Returning;
            }

            let occupied: HashSet<(usize, usize)> = snapshots.iter().map(|(_, p)| *p).collect();

            match robot.state {
                RobotState::Exploring => {
                    if robot.modules.contains(&RobotModule::Explorer) {
                        robot.smart_move(&self.map, &occupied);
                    }
                }
                RobotState::Returning => {
                    robot.step_towards(BASE_POS, &self.map, &occupied);
                }
            }

            if robot.state == RobotState::Returning && robot.position == BASE_POS {
                let _ = self.tx_report.send(robot.make_report());
                robot.state = RobotState::Exploring;
            }
        }

        while let Ok(cmd) = self.rx_cmd.try_recv() {
            match cmd {
                StationCmd::Log(line) => {
                    if self.logs.len() >= 50 {
                        self.logs.pop_front();
                    }
                    self.logs.push_back(line);
                }
                StationCmd::Spawn { modules, start_pos } => {
                    let id = self.robots.iter().map(|r| r.id).max().unwrap_or(0) + 1;
                    self.robots.push(Robot::new(id, start_pos, modules));
                }
                StationCmd::Shutdown => return true,
            }
        }

        self.tick_count > 100
    }
}
