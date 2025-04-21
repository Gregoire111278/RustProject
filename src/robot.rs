use crate::map::Tile;
use crate::station::RobotReport;
use std::collections::{HashSet, VecDeque};

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

#[derive(Debug)]
pub struct Robot {
    pub known_map: std::collections::HashMap<(usize, usize), Tile>,
    pub id: usize,
    pub position: (usize, usize),
    pub last_position: Option<(usize, usize)>,
    pub modules: Vec<RobotModule>,
    pub energy_collected: u32,
    pub mineral_collected: u32,
    pub state: RobotState,
    pub dirty_tiles: Vec<((usize, usize), Tile)>,
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
                    self.dirty_tiles.push(((r, c), tile));
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

    pub fn make_report(&mut self) -> RobotReport {
        let report = RobotReport {
            robot_id: self.id,
            map_diff: std::mem::take(&mut self.dirty_tiles),
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


// ##################################### UNIT TESTS #############################################

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::{Map, Tile};



    // TEST ROBOT CREATION -------------------------------------------------------------------------------------
    #[test]
    fn test_robot_creation() { 
        let robot_id = 1;
        let start_pos = (5, 10);
        let modules = vec![RobotModule::Explorer, RobotModule::Collector];

        let robot = Robot::new(robot_id, start_pos, modules.clone());

        assert_eq!(robot.id, robot_id);
        assert_eq!(robot.position, start_pos);
        assert_eq!(robot.last_position, None);
        assert_eq!(robot.modules, modules);
        assert_eq!(robot.energy_collected, 0);
        assert_eq!(robot.mineral_collected, 0);
        assert_eq!(robot.state, RobotState::Exploring);
        assert!(robot.known_map.is_empty());
        assert!(robot.dirty_tiles.is_empty());
    }

    fn create_test_map() -> Map {
        let grid = vec![
            vec![Tile::Energy, Tile::Mineral, Tile::Obstacle, Tile::Empty, Tile::Science],
            vec![Tile::Empty,  Tile::Energy,  Tile::Mineral, Tile::Empty, Tile::Obstacle],
            vec![Tile::Empty,  Tile::Science, Tile::Empty,   Tile::Energy, Tile::Empty],
            vec![Tile::Mineral, Tile::Empty,  Tile::Obstacle, Tile::Empty, Tile::Science],
            vec![Tile::Empty,  Tile::Empty,   Tile::Empty,   Tile::Empty, Tile::Empty],
        ];
        Map { grid, cols: 5 }
    }



    // TEST SCAN SURROUNDINGS -------------------------------------------------------------------------------------
    #[test]
    fn test_scan_surroundings_center() {
        let map = create_test_map();
        let mut robot = Robot::new(1, (2, 2), vec![RobotModule::Explorer]);
        robot.scan_surroundings(&map);
        let expected = vec![
            ((1, 1), Tile::Energy),
            ((1, 2), Tile::Mineral),
            ((1, 3), Tile::Empty),
            ((2, 1), Tile::Science),
            ((2, 2), Tile::Empty),
            ((2, 3), Tile::Energy),
            ((3, 1), Tile::Empty),
            ((3, 2), Tile::Obstacle),
            ((3, 3), Tile::Empty),
        ];

        assert_eq!(robot.known_map.len(), 9);
        for (pos, expected_tile) in expected.iter() {
            assert_eq!(robot.known_map.get(pos), Some(expected_tile), "Wrong or missing tile at {:?}", pos);
        }

        assert_eq!(robot.dirty_tiles.len(), 9);
        for (pos, expected_tile) in expected {
            assert!(robot.dirty_tiles.contains(&(pos, expected_tile)), "Missing dirty tile at {:?}", pos);
        }
    }

    #[test]
    fn test_scan_surroundings_top_left_corner() {
        let map = create_test_map();
        let mut robot = Robot::new(2, (0, 0), vec![RobotModule::Collector]);

        robot.scan_surroundings(&map);

        let expected = vec![
            ((0, 0), Tile::Energy),
            ((0, 1), Tile::Mineral),
            ((1, 0), Tile::Empty),
            ((1, 1), Tile::Energy),
        ];

        assert_eq!(robot.known_map.len(), 4);
        assert_eq!(robot.dirty_tiles.len(), 4);

        for (pos, tile) in expected {
            assert_eq!(robot.known_map.get(&pos), Some(&tile));
            assert!(robot.dirty_tiles.contains(&(pos, tile)));
        }
    }

    #[test]
    fn test_scan_surroundings_bottom_right_corner() {
        let map = create_test_map();
        let mut robot = Robot::new(3, (4, 4), vec![RobotModule::Collector]);

        robot.scan_surroundings(&map);

        let expected = vec![
            ((3, 3), Tile::Empty),
            ((3, 4), Tile::Science),
            ((4, 3), Tile::Empty),
            ((4, 4), Tile::Empty),
        ];

        assert_eq!(robot.known_map.len(), 4);
        assert_eq!(robot.dirty_tiles.len(), 4);

        for (pos, tile) in expected {
            assert_eq!(robot.known_map.get(&pos), Some(&tile));
            assert!(robot.dirty_tiles.contains(&(pos, tile)));
        }
    }



    //TEST SMART MOVE -------------------------------------------------------------------------------------
    #[test]
    fn test_robot_moves_to_nearest_resource() {
        let map = create_test_map();
        let mut robot = Robot::new(1, (4, 4), vec![RobotModule::Explorer]);
        let occupied = HashSet::new();
        robot.smart_move(&map, &occupied);
        assert_ne!(robot.position, (4, 4), "Robot should have moved");
    }

    #[test]
    fn test_robot_avoids_obstacles() {
        let map = create_test_map();
        let mut robot = Robot::new(1, (0, 1), vec![RobotModule::Explorer]);
        let occupied = HashSet::new();
        robot.smart_move(&map, &occupied);
        assert_ne!(robot.position, (0, 2), "Robot should not move into an obstacle");
    }

    #[test]
    fn test_robot_avoids_occupied_tiles() {
        let map = create_test_map();
        let mut robot = Robot::new(1, (4, 4), vec![RobotModule::Explorer]);
        let mut occupied = HashSet::new();
        occupied.insert((3, 4));
        robot.smart_move(&map, &occupied);
        assert_ne!(robot.position, (3, 4), "Robot should avoid occupied tile");
    }

    #[test]
    fn test_robot_does_not_move_if_surrounded() {
        let map = Map {
            grid: vec![
                vec![Tile::Obstacle, Tile::Obstacle, Tile::Obstacle],
                vec![Tile::Obstacle, Tile::Empty,    Tile::Obstacle],
                vec![Tile::Obstacle, Tile::Obstacle, Tile::Obstacle],
            ],
            cols: 3,
        };
        let mut robot = Robot::new(1, (1, 1), vec![RobotModule::Explorer]);
        let occupied = HashSet::new();
        robot.smart_move(&map, &occupied);
        assert_eq!(robot.position, (1, 1), "Robot should not move if surrounded");
    }


    // TEST SCAN FOR ROBOTS -------------------------------------------------------------------------------------
    fn create_robot_at(id: usize, position: (usize, usize)) -> Robot {
        Robot::new(id, position, vec![RobotModule::Explorer])
    }
    
    #[test]
    fn test_no_robots_nearby() {
        let robot = create_robot_at(1, (2, 2));
        let other_robots = vec![
            (2, (0, 0)), 
            (3, (0, 1)), 
            (4, (1, 0)),
            (7, (4, 4)), 
            (8, (5, 5)), 
            (9, (6, 6)), 
            (10, (7, 7)), 
            (3, (4, 4)), 
        ];
        let nearby = robot.scan_for_robots(&other_robots);
        println!("Nearby robots: {:?}", nearby);
        assert!(nearby.is_empty(), "No robots should be detected nearby.");
    }
    
    #[test]
    fn test_robot_detects_all_adjacent_robots() {
        let robot = create_robot_at(1, (2, 2));
        let mut snapshots = Vec::new();
        let surrounding_coords = vec![
            (1, 1), (1, 2), (1, 3),
            (2, 1),         (2, 3),
            (3, 1), (3, 2), (3, 3),
        ];
    
        for (i, pos) in surrounding_coords.iter().enumerate() {
            snapshots.push((i + 2, *pos)); // we do not push i=1
        }
        let nearby = robot.scan_for_robots(&snapshots);
        let expected: HashSet<_> = surrounding_coords.into_iter().collect();
        assert_eq!(nearby, expected);
    }
    
    #[test]
    fn test_robot_does_not_detect_itself() {
        let robot = create_robot_at(1, (2, 2));
        let snapshots = vec![
            (1, (2, 1)), 
            (2, (2, 3)), 
        ];
        let nearby = robot.scan_for_robots(&snapshots);
        assert_eq!(nearby.len(), 1);
        assert!(nearby.contains(&(2, 3)));
        assert!(!nearby.contains(&(2, 1)), "Robot should not detect itself.");
    }
    
    #[test]
    fn test_edge_wrapping_is_handled() {
        let robot = create_robot_at(1, (0, 0));
        let snapshots = vec![
            (2, (0, 1)),
            (3, (1, 0)),
            (4, (1, 1)),
        ];
        let nearby = robot.scan_for_robots(&snapshots);
        let expected: HashSet<_> = vec![(0, 1), (1, 0), (1, 1)].into_iter().collect();
        assert_eq!(nearby, expected);
    }



    // TEST MAKE REPORT -------------------------------------------------------------------------------------
    #[test]
    fn test_make_report_returns_correct_data_and_resets_robot() {
        let mut robot = Robot::new(42, (3, 3), vec![RobotModule::Collector]);
        robot.energy_collected = 7;
        robot.mineral_collected = 3;
        robot.dirty_tiles = vec![
            ((2, 2), Tile::Energy),
            ((3, 3), Tile::Mineral),
        ];
        let report = robot.make_report();

        assert_eq!(report.robot_id, 42);
        assert_eq!(report.energy, 7);
        assert_eq!(report.mineral, 3);
        assert_eq!(
            report.map_diff,
            vec![
                ((2, 2), Tile::Energy),
                ((3, 3), Tile::Mineral),
            ]
        );
        assert_eq!(robot.energy_collected, 0);
        assert_eq!(robot.mineral_collected, 0);
        assert!(robot.dirty_tiles.is_empty());
    }

    #[test]
    fn test_make_report_with_empty_fields() {
        let mut robot = Robot::new(5, (0, 0), vec![]);
        
        let report = robot.make_report();

        assert_eq!(report.robot_id, 5);
        assert_eq!(report.energy, 0);
        assert_eq!(report.mineral, 0);
        assert!(report.map_diff.is_empty());
    }


    // TEST STEP TOWARDS -------------------------------------------------------------------------------------
    #[test]
    fn test_step_towards_valid_move() {
        let map = create_test_map();
        let mut robot = Robot::new(1, (0, 0), vec![]);
        let target = (2, 0);
        let occupied = HashSet::new();
        robot.step_towards(target, &map, &occupied);
        assert_eq!(robot.position, (1, 0));
    }

    #[test]
    fn test_step_towards_avoids_obstacle() {
        let map = create_test_map();
        let mut robot = Robot::new(1, (0, 1), vec![]);
        let target = (0, 3); //Mycomment: Robot wants to move right, but (0,2) is an obstacle, see the test map
        let occupied = HashSet::new();
        robot.step_towards(target, &map, &occupied);
        assert_eq!(robot.position, (0, 1));
    }

    #[test]
    fn test_step_towards_avoids_occupied() {
        let map = create_test_map();
        let mut robot = Robot::new(1, (0, 0), vec![]);
        let target = (2, 0);
        let mut occupied = HashSet::new();
        occupied.insert((1, 0));
        robot.step_towards(target, &map, &occupied);
        assert_eq!(robot.position, (0, 0)); //Mycomment: robot should not move
    }

    #[test]
    fn test_step_towards_rightward_move() {
        let map = create_test_map();
        let mut robot = Robot::new(1, (0, 0), vec![]);
        let target = (0, 2);
        let occupied = HashSet::new();
        robot.step_towards(target, &map, &occupied);
        assert_eq!(robot.position, (0, 1)); //Mycomment: robot should move right
    }
}
