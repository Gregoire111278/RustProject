use std::collections::{HashMap, HashSet};
use std::sync::{mpsc, Arc, RwLock};
use std::thread;
use std::time::Duration;

use crate::map::{Map, MapDiff, Tile};
use crate::robot::{Robot, RobotActor, RobotCmd};
use crate::station::{RobotReport, StationCmd};

pub struct RobotCoordinator {
    map: Arc<RwLock<Map>>,
    robot_senders: HashMap<usize, mpsc::Sender<RobotCmd>>,
    robot_threads: HashMap<usize, thread::JoinHandle<()>>,
    tx_report: mpsc::Sender<RobotReport>,
    rx_cmd: mpsc::Receiver<StationCmd>,
    next_robot_id: usize,
}

impl RobotCoordinator {
    pub fn new(
        map: Map,
        tx_report: mpsc::Sender<RobotReport>,
        rx_cmd: mpsc::Receiver<StationCmd>,
        initial_robots: Vec<Robot>,
    ) -> Self {
        let map = Arc::new(RwLock::new(map));
        let mut coordinator = Self {
            map,
            robot_senders: HashMap::new(),
            robot_threads: HashMap::new(),
            tx_report,
            rx_cmd,
            next_robot_id: initial_robots.iter().map(|r| r.id).max().unwrap_or(0) + 1,
        };

        for robot in initial_robots {
            coordinator.spawn_robot_actor(robot);
        }

        coordinator
    }

    fn spawn_robot_actor(&mut self, robot: Robot) {
        let robot_id = robot.id;
        let (tx, rx) = mpsc::channel();
        let tx_report = self.tx_report.clone();
        let map_clone = Arc::clone(&self.map);

        let _ = self.tx_report.send(RobotReport {
            robot_id,
            tick: 0,
            map_diff: crate::map::MapDiff(vec![]),
            energy: 0,
            mineral: 0,
        });

        let actor = RobotActor::new(robot, map_clone, rx, tx_report);
        let handle = thread::spawn(move || {
            actor.run();
        });

        self.robot_senders.insert(robot_id, tx);
        self.robot_threads.insert(robot_id, handle);
    }

    pub fn tick(&mut self, tick_count: u64) -> (bool, Vec<(usize, (usize, usize))>) {
        let mut done = false;

        while let Ok(cmd) = self.rx_cmd.try_recv() {
            match cmd {
                StationCmd::Spawn {
                    id,
                    modules,
                    start_pos,
                } => {
                    let robot = Robot::new(id, start_pos, modules);
                    self.spawn_robot_actor(robot);
                }
                StationCmd::Snapshot { id, version, diff } => {
                    if let Ok(mut map) = self.map.write() {
                        diff.apply(&mut map);
                    }

                    if let Some(tx) = self.robot_senders.get(&(id as usize)) {
                        let _ = tx.send(RobotCmd::Snapshot { version, diff });
                    }
                }
                StationCmd::Shutdown => {
                    self.shutdown();
                    done = true;
                }
                _ => {}
            }
        }

        let (tx_pos1, rx_pos1) = mpsc::channel();
        for (id, tx) in &self.robot_senders {
            let tx_pos_clone = tx_pos1.clone();
            let _ = tx.send(RobotCmd::ReportPosition {
                respond_to: tx_pos_clone,
            });
        }

        let mut current_positions = Vec::new();
        for _ in 0..self.robot_senders.len() {
            if let Ok((id, pos)) = rx_pos1.recv_timeout(Duration::from_millis(50)) {
                current_positions.push((id, pos));
            }
        }

        let mut total_energy = 0;
        let mut total_mineral = 0;
        let mut all_map_updates = Vec::new();

        let mut robot_collections = HashMap::<usize, (u32, u32)>::new();

        {
            let mut map = self.map.write().unwrap();

            for &(robot_id, (row, col)) in &current_positions {
                if row < map.grid.len() && col < map.cols {
                    let tile = map.grid[row][col];
                    if matches!(tile, Tile::Energy | Tile::Mineral) {
                        map.grid[row][col] = Tile::Empty;

                        all_map_updates.push(((row, col), Some(tile), Tile::Empty));

                        if tile == Tile::Energy {
                            total_energy += 1;
                            let entry = robot_collections.entry(robot_id).or_insert((0, 0));
                            entry.0 += 1;
                        } else if tile == Tile::Mineral {
                            total_mineral += 1;
                            let entry = robot_collections.entry(robot_id).or_insert((0, 0));
                            entry.1 += 1;
                        }
                    }
                }
            }
        }

        if !all_map_updates.is_empty() {
            let _ = self.tx_report.send(RobotReport {
                robot_id: 0,
                tick: tick_count,
                map_diff: MapDiff(all_map_updates),
                energy: total_energy,
                mineral: total_mineral,
            });
        }

        let map_obstacles: HashSet<(usize, usize)> = {
            let map = self.map.read().unwrap();
            map.grid
                .iter()
                .enumerate()
                .flat_map(|(r, row)| {
                    row.iter().enumerate().filter_map(move |(c, tile)| {
                        if tile == &Tile::Obstacle {
                            Some((r, c))
                        } else {
                            None
                        }
                    })
                })
                .collect()
        };

        for (id, tx) in &self.robot_senders {
            let mut occupied = map_obstacles.clone();
            for &(robot_id, pos) in &current_positions {
                if robot_id != *id {
                    occupied.insert(pos);
                }
            }

            let _ = tx.send(RobotCmd::Tick {
                tick_count,
                occupied_positions: occupied,
            });
        }

        std::thread::sleep(Duration::from_millis(20));

        let (tx_pos2, rx_pos2) = mpsc::channel();
        for (id, tx) in &self.robot_senders {
            let tx_pos_clone = tx_pos2.clone();
            let _ = tx.send(RobotCmd::ReportPosition {
                respond_to: tx_pos_clone,
            });
        }

        let mut final_positions = Vec::new();
        for _ in 0..self.robot_senders.len() {
            if let Ok((id, pos)) = rx_pos2.recv_timeout(Duration::from_millis(50)) {
                final_positions.push((id, pos));
            }
        }

        (done, final_positions)
    }

    pub fn shutdown(&mut self) {
        for tx in self.robot_senders.values() {
            let _ = tx.send(RobotCmd::Shutdown);
        }

        let threads = std::mem::take(&mut self.robot_threads);
        for (_, handle) in threads {
            let _ = handle.join();
        }
    }
}
