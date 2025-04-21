use rust_project::station::{RobotReport, Station, StationCmd};
use rust_project::map::{Tile, MapDiff};
use rust_project::robot::RobotModule;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;


#[test]
fn test_station_report_processing_and_spawning() {
    let (tx_report, rx_report): (Sender<RobotReport>, Receiver<RobotReport>) = mpsc::channel();
    let (tx_cmd, rx_cmd): (Sender<StationCmd>, Receiver<StationCmd>) = mpsc::channel();

    let handle = thread::spawn(move || {
        let mut station = Station::new(rx_report, tx_cmd);
        station.run();
    });

    let map_diff = MapDiff(vec![
        ((0, 0), Some(Tile::Empty), Tile::Energy),
        ((0, 1), Some(Tile::Empty), Tile::Mineral),
    ]);

    tx_report
        .send(RobotReport {
            robot_id: 1,
            tick: 1,
            map_diff,
            energy: 10,
            mineral: 10,
        })
        .unwrap();

    drop(tx_report); 

    let mut spawn_found = false;
    let mut log_found = false;

    for _ in 0..3 {
        if let Ok(msg) = rx_cmd.recv() {
            match msg {
                StationCmd::Log(log) => {
                    println!("Log: {log}");
                    if log.contains("Merged") {
                        log_found = true;
                    }
                }
                StationCmd::Spawn { id, modules, start_pos } => {
                    assert_eq!(modules, vec![
                        RobotModule::Explorer,
                        RobotModule::Collector,
                        RobotModule::Scanner,
                        RobotModule::Sensor,
                    ]);
                    assert_eq!(start_pos, (0, 0));
                    assert_eq!(id, 3);
                    spawn_found = true;
                }
                _ => {}
            }
        }
    }

    handle.join().unwrap();

    assert!(log_found, "Expected log not found");
    assert!(spawn_found, "Expected robot spawn command not found");
}

#[test]
fn test_station_does_not_spawn_with_insufficient_resources() {
    let (tx_report, rx_report): (Sender<RobotReport>, Receiver<RobotReport>) = mpsc::channel();
    let (tx_cmd, rx_cmd): (Sender<StationCmd>, Receiver<StationCmd>) = mpsc::channel();

    let handle = thread::spawn(move || {
        let mut station = Station::new(rx_report, tx_cmd);
        station.run();
    });

    let map_diff = MapDiff(vec![((1, 1), Some(Tile::Empty), Tile::Science)]);

    tx_report
        .send(RobotReport {
            robot_id: 2,
            tick: 1,
            map_diff,
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
