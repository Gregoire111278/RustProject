use crate::map::{MapDiff, Tile};
use crate::robot::RobotModule;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};

#[derive(Debug, Clone)]
pub struct RobotReport {
    pub robot_id: usize,
    pub tick: u64,
    pub map_diff: MapDiff,
    pub energy: u32,
    pub mineral: u32,
}

#[derive(Debug)]
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
    pending: Vec<RobotReport>,
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
            pending: Vec::new(),
            energy_stock: 0,
            mineral_stock: 0,
            next_robot_id: 3,
        }
    }

    pub fn run(&mut self) {
        while let Ok(report) = self.rx.recv() {
            self.pending.push(report);
            self.merge_pending_diffs();

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
            }
        }
    }

    fn merge_pending_diffs(&mut self) {
        let Some(min_tick) = self.pending.iter().map(|r| r.tick).min() else {
            return;
        };

        let mut same_tick: Vec<RobotReport> = Vec::new();
        self.pending.retain(|r| {
            if r.tick == min_tick {
                same_tick.push(r.clone());
                false
            } else {
                true
            }
        });

        if same_tick.is_empty() {
            return;
        }

        let mut cell_updates: HashMap<(usize, usize), (usize, Tile)> = HashMap::new();
        for (arrival_idx, rep) in same_tick.iter().enumerate() {
            for &((row, col), _before, after) in &rep.map_diff.0 {
                let entry = cell_updates
                    .entry((row, col))
                    .or_insert((arrival_idx, after));
                if arrival_idx > entry.0 {
                    *entry = (arrival_idx, after);
                }
            }
            self.energy_stock += rep.energy;
            self.mineral_stock += rep.mineral;
        }

        for ((row, col), &(_, tile_after)) in &cell_updates {
            self.master_map.insert((*row, *col), tile_after);
        }

        let _ = self.tx_cmd.send(StationCmd::Log(format!(
            "Merged {} diffs (tick {}) | stocks {}E {}M",
            same_tick.len(),
            min_tick,
            self.energy_stock,
            self.mineral_stock
        )));
    }
}
