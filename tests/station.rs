use RustProject::station::{RobotReport, Station, StationCmd};
use RustProject::map::Tile;
use RustProject::robot::RobotModule;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::mpsc;
use std::thread;



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