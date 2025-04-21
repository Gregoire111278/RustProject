use crate::coordinator::RobotCoordinator;
use crate::map::{Map, Tile};
use crate::robot::{Robot, RobotModule};
use crate::station;
use crate::station::StationCmd;
use std::collections::VecDeque;
use std::sync::mpsc;

pub struct App {
    pub map: Map,
    pub robots: Vec<Robot>,
    pub tick_count: u64,
    pub collected_energy: u32,
    pub collected_mineral: u32,
    tx_report: mpsc::Sender<station::RobotReport>,
    rx_cmd: mpsc::Receiver<StationCmd>,
    pub logs: VecDeque<String>,
    pub robots_scroll: u16,
    pub logs_scroll: u16,
    pub master_version: u64,
    coordinator: RobotCoordinator,
    tx_coord_cmd: mpsc::Sender<StationCmd>,
}

impl App {
    pub fn new(
        tx_report: mpsc::Sender<station::RobotReport>,
        rx_cmd: mpsc::Receiver<StationCmd>,
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

        let (tx_coord_cmd, rx_coord_cmd) = mpsc::channel();

        let coordinator =
            RobotCoordinator::new(map.clone(), tx_report.clone(), rx_coord_cmd, robots.clone());

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
            master_version: 0,
            coordinator,
            tx_coord_cmd,
        }
    }

    pub fn update_map_tile(&mut self, row: usize, col: usize, tile: Tile) {
        if row < self.map.grid.len() && col < self.map.cols {
            self.map.grid[row][col] = tile;
        }
    }

    pub fn tick(&mut self) -> bool {
        self.tick_count += 1;

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
                    self.robots.push(Robot::new(id, start_pos, modules.clone()));

                    let _ = self.tx_coord_cmd.send(StationCmd::Spawn {
                        id,
                        modules,
                        start_pos,
                    });
                }
                StationCmd::Snapshot { id, version, diff } => {
                    self.master_version = version;

                    for &((row, col), _, new_tile) in &diff.0 {
                        self.update_map_tile(row, col, new_tile);
                    }

                    if let Some(robot) = self.robots.iter_mut().find(|r| r.id == id as usize) {
                        diff.apply_to_known_map(&mut robot.known_map);
                    }

                    let _ = self.tx_coord_cmd.send(StationCmd::Snapshot {
                        id,
                        version,
                        diff: diff.clone(),
                    });
                }
                StationCmd::Version(v) => {
                    self.master_version = v;
                }
                StationCmd::ResourceUpdate { energy, mineral } => {
                    self.collected_energy += energy;
                    self.collected_mineral += mineral;
                }
                StationCmd::Shutdown => {
                    let _ = self.tx_coord_cmd.send(StationCmd::Shutdown);
                    return true;
                }
            }
        }

        let (done, positions) = self.coordinator.tick(self.tick_count);

        for (id, position) in positions {
            if let Some(robot) = self.robots.iter_mut().find(|r| r.id == id) {
                robot.position = position;
            }
        }

        self.tick_count > 200 || done
    }
}
