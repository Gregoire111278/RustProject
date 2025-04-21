use crate::map::Tile;
use crate::robot::RobotModule;
use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

#[derive(Debug, Clone)]
pub struct RobotReport {
    pub robot_id: usize,
    pub map_diff: Vec<((usize, usize), Tile)>,
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
            for (coord, tile) in report.map_diff {
                self.master_map.insert(coord, tile);
            }
            self.energy_stock += report.energy;
            self.mineral_stock += report.mineral;

            let _ = self.tx_cmd.send(StationCmd::Log(format!(
                "Robot #{:?} delivered {}E {}M  | stocks {}E {}M",
                report.robot_id, report.energy, report.mineral, self.energy_stock, self.mineral_stock
            )));
            
            if self.energy_stock >= 10 && self.mineral_stock >= 10 {
                self.energy_stock -= 10;
                self.mineral_stock -= 10;

                let _ = self.tx_cmd.send(StationCmd::Log("Spawning new robot".into()));
                let _ = self.tx_cmd.send(StationCmd::Spawn {
                    modules: vec![RobotModule::Explorer, RobotModule::Collector],
                    start_pos: (0, 0),
                });
            }
        }
    }
}


// ##################################### UNIT TESTS #############################################
#[test]
fn test_station_report_processing_and_spawning() {

    let (tx_report,
         rx_report):
            (Sender<RobotReport>, Receiver<RobotReport>) = mpsc::channel();

    let (tx_cmd,
         rx_cmd):
            (Sender<StationCmd>, Receiver<StationCmd>) = mpsc::channel();

    
    let handle = thread::spawn(move || {
        let station = Station::new(rx_report, tx_cmd);
        station.run();
    });

  
    tx_report
        .send(RobotReport {
            robot_id: 1,
            map_diff: vec![
                ((0, 0), Tile::Energy),
                ((0, 1), Tile::Mineral),
            ],
            energy: 10,
            mineral: 10,
        })
        .unwrap();

    drop(tx_report);

    let mut log_found = false;
    let mut spawn_found = false;

    for _ in 0..3 {
        if let Ok(msg) = rx_cmd.recv() {
            match msg {
                StationCmd::Log(log) => {
                    println!("Log: {log}");
                    if log.contains("Robot #1 delivered") {
                        log_found = true;
                    }
                }
                StationCmd::Spawn { modules, start_pos } => {
                    assert_eq!(modules, vec![RobotModule::Explorer, RobotModule::Collector]);
                    assert_eq!(start_pos, (0, 0));
                    spawn_found = true;
                }
                _ => {}
            }
        }
    }

    handle.join().unwrap();

    assert!(log_found, "Expected delivery log not found");
    assert!(spawn_found, "Expected robot spawn command not found");
}



#[test]
fn test_station_does_not_spawn_with_insufficient_resources() {

    let (tx_report,
         rx_report): 
            (Sender<RobotReport>, Receiver<RobotReport>) = mpsc::channel();

    let (tx_cmd,
         rx_cmd): 
            (Sender<StationCmd>, Receiver<StationCmd>) = mpsc::channel();

    let handle = thread::spawn(move || {
        let station = Station::new(rx_report, tx_cmd);
        station.run();
    });

    tx_report
        .send(RobotReport {
            robot_id: 2,
            map_diff: vec![((1, 1), Tile::Science)],
            energy: 5,
            mineral: 3,
        })
        .unwrap();

    drop(tx_report); 

    let mut received_spawn = false;

    for _ in 0..2 {
        if let Ok(cmd) = rx_cmd.recv() {
            if let StationCmd::Spawn { .. } = cmd {
                received_spawn = true;
            }
        }
    }

    handle.join().unwrap();

    assert!(!received_spawn, "Robot was spawned with insufficient resources");

}