use crate::map::Tile;
use crate::robot::RobotModule;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};

#[derive(Debug)]
pub struct RobotReport {
    pub robot_id: usize,
    pub map_diff: Vec<((usize, usize), Tile)>,
    pub energy: u32,
    pub mineral: u32,
}

#[derive(Debug)]
pub enum StationCmd {
    Spawn {
        modules: Vec<RobotModule>,
        start_pos: (usize, usize),
    },
    Shutdown,
}

pub struct Station {
    rx: Receiver<RobotReport>,
    tx_cmd: Sender<StationCmd>,

    master_map: HashMap<(usize, usize), Tile>,
    energy_stock: u32,
    mineral_stock: u32,
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
            println!("STATION received: {:?}", report);

            for (coord, tile) in report.map_diff {
                self.master_map.insert(coord, tile);
            }
            self.energy_stock += report.energy;
            self.mineral_stock += report.mineral;

            if self.energy_stock >= 10 && self.mineral_stock >= 10 {
                self.energy_stock -= 10;
                self.mineral_stock -= 10;

                let _ = self.tx_cmd.send(StationCmd::Spawn {
                    modules: vec![RobotModule::Explorer, RobotModule::Collector],
                    start_pos: (0, 0),
                });
                self.next_robot_id += 1;
            }
        }
        println!("STATION shutting down");
    }
}
