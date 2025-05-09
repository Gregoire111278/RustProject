use crate::map::Tile;
use crate::map::{Map, MapDiff};
use crate::station::RobotReport;
use std::collections::{HashSet, VecDeque};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, RwLock};

/// nbr of resources a robot can carry
pub const PAYLOAD_LIMIT: u32 = 10;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum RobotState {
    Exploring,
    Returning,
}

#[derive(Debug, PartialEq, Clone)]
pub enum RobotModule {
    Explorer,
    Collector,
    Scanner,
    Sensor,
}

pub enum RobotCmd {
    Tick {
        tick_count: u64,
        occupied_positions: HashSet<(usize, usize)>,
    },
    Snapshot {
        version: u64,
        diff: MapDiff,
    },
    ReportPosition {
        respond_to: Sender<(usize, (usize, usize))>,
    },
    Shutdown,
}

pub struct RobotActor {
    robot: Robot,
    map: Arc<RwLock<Map>>,
    rx: Receiver<RobotCmd>,
    tx_report: Sender<RobotReport>,
}

impl RobotActor {
    pub fn new(
        robot: Robot,
        map: Arc<RwLock<Map>>,
        rx: Receiver<RobotCmd>,
        tx_report: Sender<RobotReport>,
    ) -> Self {
        Self {
            robot,
            map,
            rx,
            tx_report,
        }
    }

    pub fn run(mut self) {
        while let Ok(cmd) = self.rx.recv() {
            match cmd {
                RobotCmd::Tick {
                    tick_count,
                    occupied_positions,
                } => {
                    self.process_tick(tick_count, &occupied_positions);
                }
                RobotCmd::Snapshot { version: _, diff } => {
                    diff.apply_to_known_map(&mut self.robot.known_map);
                }
                RobotCmd::ReportPosition { respond_to } => {
                    let _ = respond_to.send((self.robot.id, self.robot.position));
                }
                RobotCmd::Shutdown => break,
            }
        }
    }

    fn process_tick(&mut self, tick_count: u64, occupied: &HashSet<(usize, usize)>) {
        let map = self.map.read().unwrap();

        if self.robot.modules.contains(&RobotModule::Scanner) {
            self.robot.scan_surroundings(&map);
        }

        if self.robot.modules.contains(&RobotModule::Collector) {
            let (row, col) = self.robot.position;
            if row < map.grid.len() && col < map.cols {
                let tile = map.grid[row][col];
                if matches!(tile, Tile::Energy | Tile::Mineral) {
                    self.robot
                        .dirty_tiles
                        .push(((row, col), Some(tile), Tile::Empty));

                    if tile == Tile::Energy {
                        self.robot.energy_collected += 1;
                    } else if tile == Tile::Mineral {
                        self.robot.mineral_collected += 1;
                    }
                }
            }
        }

        if self.robot.state == RobotState::Exploring
            && self.robot.energy_collected + self.robot.mineral_collected >= PAYLOAD_LIMIT
        {
            self.robot.state = RobotState::Returning;
        }

        match self.robot.state {
            RobotState::Exploring => {
                if self.robot.modules.contains(&RobotModule::Explorer) {
                    self.robot.smart_move(&map, occupied);
                }
            }
            RobotState::Returning => {
                self.robot.step_towards((0, 0), &map, occupied);
            }
        }

        if self.robot.state == RobotState::Returning && self.robot.position == (0, 0) {
            let report = self.robot.make_report(tick_count);
            let _ = self.tx_report.send(report);
            self.robot.state = RobotState::Exploring;
        }

        drop(map);
    }
}

#[derive(Debug, Clone)]
pub struct Robot {
    pub known_map: std::collections::HashMap<(usize, usize), Tile>,
    pub id: usize,
    pub position: (usize, usize),
    pub last_position: Option<(usize, usize)>,
    pub modules: Vec<RobotModule>,
    pub energy_collected: u32,
    pub mineral_collected: u32,
    pub state: RobotState,
    pub dirty_tiles: Vec<((usize, usize), Option<Tile>, Tile)>,
}

impl Robot {
    pub fn new(id: usize, position: (usize, usize), modules: Vec<RobotModule>) -> Self {
        Self {
            known_map: std::collections::HashMap::new(),
            id,
            position,
            last_position: None,
            modules,
            energy_collected: 0,
            mineral_collected: 0,
            state: RobotState::Exploring,
            dirty_tiles: Vec::new(),
        }
    }

    pub fn scan_surroundings(&mut self, map: &crate::map::Map) {
        let (row, col) = self.position;
        for dr in -1..=1 {
            for dc in -1..=1 {
                let r = row.wrapping_add(dr as usize);
                let c = col.wrapping_add(dc as usize);
                if r < map.grid.len() && c < map.cols {
                    let tile = map.grid[r][c];
                    self.known_map.insert((r, c), tile);
                    self.dirty_tiles.push(((r, c), None, tile));
                }
            }
        }
    }

    pub fn smart_move(&mut self, map: &crate::map::Map, occupied: &HashSet<(usize, usize)>) {
        let (sr, sc) = self.position;

        let mut q = VecDeque::new();
        let mut visited = HashSet::new();
        let mut parent = std::collections::HashMap::new();

        q.push_back((sr, sc));
        visited.insert((sr, sc));

        let target = 'search: loop {
            while let Some((r, c)) = q.pop_front() {
                if matches!(map.grid[r][c], Tile::Energy | Tile::Mineral) && (r, c) != (sr, sc) {
                    break 'search Some((r, c));
                }

                for (dr, dc) in &[(0, 1), (1, 0), (0, usize::MAX), (usize::MAX, 0)] {
                    let nr = r.wrapping_add(*dr);
                    let nc = c.wrapping_add(*dc);

                    if nr >= map.grid.len()
                        || nc >= map.cols
                        || visited.contains(&(nr, nc))
                        || occupied.contains(&(nr, nc))
                        || matches!(map.grid[nr][nc], Tile::Obstacle)
                    {
                        continue;
                    }
                    visited.insert((nr, nc));
                    parent.insert((nr, nc), (r, c));
                    q.push_back((nr, nc));
                }
            }
            break None;
        };

        let next = if let Some(mut cur) = target {
            while let Some(&p) = parent.get(&cur) {
                if p == (sr, sc) {
                    break;
                }
                cur = p;
            }
            Some(cur)
        } else {
            let dirs = [(0, 1), (1, 0), (0, usize::MAX), (usize::MAX, 0)];
            let mut best = None;
            let mut best_score = -1;
            for &(dr, dc) in &dirs {
                let r = sr.wrapping_add(dr);
                let c = sc.wrapping_add(dc);
                if r >= map.grid.len()
                    || c >= map.cols
                    || occupied.contains(&(r, c))
                    || matches!(map.grid[r][c], Tile::Obstacle)
                {
                    continue;
                }
                let score = if !self.known_map.contains_key(&(r, c)) {
                    2
                } else {
                    1
                };
                if score > best_score {
                    best_score = score;
                    best = Some((r, c));
                }
            }
            best
        };

        if let Some(p) = next {
            self.last_position = Some(self.position);
            self.position = p;
        }
    }

    #[allow(dead_code)]
    pub fn scan_for_robots(
        &self,
        robot_snapshots: &[(usize, (usize, usize))],
    ) -> HashSet<(usize, usize)> {
        let mut nearby = HashSet::new();
        let (row, col) = self.position;

        for dr in -1..=1 {
            for dc in -1..=1 {
                let r = row.wrapping_add(dr as usize);
                let c = col.wrapping_add(dc as usize);
                if (r, c) != self.position {
                    if robot_snapshots
                        .iter()
                        .any(|&(id, pos)| id != self.id && pos == (r, c))
                    {
                        nearby.insert((r, c));
                    }
                }
            }
        }

        nearby
    }

    pub fn make_report(&mut self, tick: u64) -> RobotReport {
        let diff_vec = std::mem::take(&mut self.dirty_tiles);
        let report = RobotReport {
            robot_id: self.id,
            tick,
            map_diff: MapDiff(diff_vec),
            energy: std::mem::take(&mut self.energy_collected),
            mineral: std::mem::take(&mut self.mineral_collected),
        };
        report
    }

    pub fn step_towards(
        &mut self,
        target: (usize, usize),
        map: &crate::map::Map,
        occupied: &HashSet<(usize, usize)>,
    ) {
        let (tr, tc) = target;
        let mut candidates = Vec::new();
        if self.position.0 > tr {
            candidates.push((-1isize, 0));
        }
        if self.position.0 < tr {
            candidates.push((1, 0));
        }
        if self.position.1 > tc {
            candidates.push((0, -1));
        }
        if self.position.1 < tc {
            candidates.push((0, 1));
        }

        for (dr, dc) in candidates {
            let nr = self.position.0.wrapping_add(dr as usize);
            let nc = self.position.1.wrapping_add(dc as usize);
            if nr < map.grid.len()
                && nc < map.cols
                && !occupied.contains(&(nr, nc))
                && map.grid[nr][nc] != Tile::Obstacle
            {
                self.position = (nr, nc);
                break;
            }
        }
    }
}
