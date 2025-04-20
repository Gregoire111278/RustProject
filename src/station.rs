use crate::map::{MapDiff, Tile};
use crate::robot::RobotModule;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};

#[derive(Debug, Clone)]
pub struct RobotReport {
    pub robot_id: usize,
    pub map_diff: MapDiff,
    pub energy: u32,
    pub mineral: u32,
}

#[derive(Debug, Clone)]
pub enum StationCmd {
    Log(String),
    Spawn {
        id: usize,
        modules: Vec<RobotModule>,
        start_pos: (usize, usize),
    },
    Shutdown,
}

pub struct Station {
    rx: Receiver<RobotReport>,
    tx_cmd: Sender<StationCmd>,

    pub master_map: HashMap<(usize, usize), Tile>,
    pub energy_stock: u32,
    pub mineral_stock: u32,

    pending: Vec<RobotReport>,
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
            pending: Vec::new(),
            next_robot_id: 3,
        }
    }

    pub fn run(mut self) {
        while let Ok(report) = self.rx.recv() {
            self.pending.push(report);

            while let Some(r) = self.pending.pop() {
                for &((r_idx, c_idx), _before, after) in &r.map_diff.0 {
                    self.master_map.insert((r_idx, c_idx), after);
                }
                self.energy_stock += r.energy;
                self.mineral_stock += r.mineral;

                let _ = self.tx_cmd.send(StationCmd::Log(format!(
                    "Robot #{:?} delivered {}E {}M | stocks {}E {}M",
                    r.robot_id, r.energy, r.mineral, self.energy_stock, self.mineral_stock
                )));
            }

            if self.energy_stock >= 10 && self.mineral_stock >= 10 {
                self.energy_stock -= 10;
                self.mineral_stock -= 10;

                let id = self.next_robot_id;
                self.next_robot_id += 1;

                let _ = self.tx_cmd.send(StationCmd::Spawn {
                    id,
                    modules: vec![
                        RobotModule::Explorer,
                        RobotModule::Collector,
                        RobotModule::Scanner,
                        RobotModule::Sensor,
                    ],
                    start_pos: (0, 0),
                });
                let _ = self
                    .tx_cmd
                    .send(StationCmd::Log("Spawning new robot".to_string()));
            }
        }
    }
}
