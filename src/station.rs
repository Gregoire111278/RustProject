use crate::map::Tile;
use crate::robot::RobotModule;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};

#[derive(Debug, Clone)]
pub struct RobotReport {
    pub robot_id: usize,
    pub map_diff: Vec<((usize, usize), Option<Tile>, Tile)>,
    pub energy: u32,
    pub mineral: u32,
}

#[derive(Debug, Clone)]
pub enum StationCmd {
    Spawn {
        modules: Vec<RobotModule>,
        start_pos: (usize, usize),
    },
    Log(String),
    #[allow(dead_code)]
    Shutdown,
}

pub struct Station {
    rx: Receiver<RobotReport>,
    tx_cmd: Sender<StationCmd>,
    master_map: HashMap<(usize, usize), Tile>,
    energy_stock: u32,
    mineral_stock: u32,
    #[allow(dead_code)]
    next_robot_id: usize,
}

impl Station {
    pub fn new(rx: Receiver<RobotReport>, tx_cmd: Sender<StationCmd>) -> Self {
        Self {
            rx,
            tx_cmd,
            master_map: HashMap::new(),
            energy_stock: 0,
            mineral_stock: 0,
            next_robot_id: 3,
        }
    }

    pub fn run(mut self) {
        while let Ok(report) = self.rx.recv() {
            for (coord, _old_tile, new_tile) in report.map_diff {
                self.master_map.insert(coord, new_tile);
            }
            self.energy_stock += report.energy;
            self.mineral_stock += report.mineral;

            let _ = self.tx_cmd.send(StationCmd::Log(format!(
                "Robot #{:?} delivered {}E {}M  | stocks {}E {}M",
                report.robot_id,
                report.energy,
                report.mineral,
                self.energy_stock,
                self.mineral_stock
            )));

            if self.energy_stock >= 10 && self.mineral_stock >= 10 {
                self.energy_stock -= 10;
                self.mineral_stock -= 10;

                let _ = self
                    .tx_cmd
                    .send(StationCmd::Log("Spawning new robot".into()));
                let _ = self.tx_cmd.send(StationCmd::Spawn {
                    modules: vec![RobotModule::Explorer, RobotModule::Collector],
                    start_pos: (0, 0),
                });
            }
        }
    }
}
