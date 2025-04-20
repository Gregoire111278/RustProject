use crate::map::{Map, Tile};
use crate::robot::{Robot, RobotModule, RobotState, PAYLOAD_LIMIT};
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
    pub robots_scroll: u16,
    pub logs_scroll: u16,
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
            robots_scroll: 0,
            logs_scroll: 0,
        }
    }

    fn at_station(pos: (usize, usize)) -> bool {
        pos == BASE_POS
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
                            let prev = *tile;
                            robot.energy_collected += 1;
                            self.collected_energy += 1;
                            *tile = Tile::Empty;
                            robot
                                .dirty_tiles
                                .push(((row, col), Some(prev), Tile::Empty));
                        }
                        Tile::Mineral => {
                            let prev = *tile;
                            robot.mineral_collected += 1;
                            self.collected_mineral += 1;
                            *tile = Tile::Empty;
                            robot
                                .dirty_tiles
                                .push(((row, col), Some(prev), Tile::Empty));
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

            if robot.state == RobotState::Returning && Self::at_station(robot.position) {
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
                StationCmd::Spawn {
                    id,
                    modules,
                    start_pos,
                } => {
                    self.robots.push(Robot::new(id, start_pos, modules));
                }
                StationCmd::Shutdown => return true,
            }
        }

        self.tick_count > 200
    }
}
