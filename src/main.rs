use bevy::prelude::*;
use noise::{NoiseFn, Perlin, Seedable};
use rand::Rng;
use std::collections::HashMap;

const TILE_SIZE: f32 = 4.0;
const WIDTH: usize = 370;
const HEIGHT: usize = 190;
const MINERAL_SIZE: f32 = TILE_SIZE * 10.0;
const RESOURCE_HALF_TILES: usize = (MINERAL_SIZE / TILE_SIZE / 2.0) as usize;
const ROBOT_VIEW_DISTANCE: usize = 15;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum TileType {
    Unknown,
    Passable,
    Obstacle,
    Science,
    Mineral,
    Energy,
    Station,
}

#[derive(Component)]
struct Tile {
    tile_type: TileType,
    x: usize,
    y: usize,
}

#[derive(Component, Clone, Copy)]
struct ResourcePoint {
    resource_type: ResourceType,
    x: usize,
    y: usize,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ResourceType {
    Mineral,
    Energy,
}

#[derive(Component)]
struct Station;

#[derive(Component)]
struct Robot {
    x: usize,
    y: usize,
    current_goal: RobotGoal,
    direction: Direction,
    directional_momentum: u32,
    exploration_timer: Timer,
    carrying_resource: Option<ResourceType>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Direction {
    North,
    South,
    East,
    West,
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
}

impl Direction {
    fn get_direction_vector(&self) -> (isize, isize) {
        match self {
            Direction::North => (0, 1),
            Direction::South => (0, -1),
            Direction::East => (1, 0),
            Direction::West => (-1, 0),
            Direction::NorthEast => (1, 1),
            Direction::NorthWest => (-1, 1),
            Direction::SouthEast => (1, -1),
            Direction::SouthWest => (-1, -1),
        }
    }

    fn random() -> Self {
        let directions = [
            Direction::North,
            Direction::South,
            Direction::East,
            Direction::West,
            Direction::NorthEast,
            Direction::NorthWest,
            Direction::SouthEast,
            Direction::SouthWest,
        ];
        let mut rng = rand::thread_rng();
        directions[rng.gen_range(0..directions.len())]
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum RobotGoal {
    Explore,
    GoToResource { x: usize, y: usize, resource_type: ResourceType },
    ReturnToStation,
    Idle,
}

#[derive(Component)]
struct MapMemory {
    tiles: HashMap<(usize, usize), TileType>,
    known_resources: Vec<(usize, usize, ResourceType)>,
}

#[derive(Resource)]
struct StationPosition {
    x: usize,
    y: usize,
}

#[derive(Resource)]
struct WorldMap {
    tiles: Vec<Vec<TileType>>,
}

// Compteur de ressources
#[derive(Resource, Default)]
struct ResourceCounter {
    minerals: u32,
    energy: u32,
}

// Composant pour le texte des compteurs
#[derive(Component)]
enum CounterText {
    Mineral,
    Energy,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(ResourceCounter::default())
        .add_systems(Startup, setup)
        .add_systems(Startup, setup_ui)
        .add_systems(Update, (
            robot_observation_system,
            robot_decision_system,
            robot_movement_system,
            direct_resource_collection,  // Système simplifié pour la collecte
            update_robot_visuals,        // Changement visuel des robots
            update_counter_text,         // Mise à jour des compteurs
        ).chain())
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 1000.0),
        ..Default::default()
    });

    let seed = 42;
    let perlin = Perlin::new().set_seed(seed);
    let mut rng = rand::thread_rng();

    let desert_handle = asset_server.load("desert.jpg");
    let obstacle_handle = asset_server.load("obstacle.jpg");
    let science_handle = asset_server.load("gris.jpg");
    let mineral_handle = asset_server.load("rubis.png");
    let energy_handle = asset_server.load("eclair.png");
    let station_handle = asset_server.load("station.png");
    let robot_handle = asset_server.load("robot.png");

    let mut tile_map: Vec<Vec<Handle<Image>>> =
        vec![vec![desert_handle.clone(); WIDTH]; HEIGHT];
    let mut world_map = vec![vec![TileType::Passable; WIDTH]; HEIGHT];

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let value = perlin.get([x as f64 / 5.0, y as f64 / 5.0]);
            if value < 0.50 {
                tile_map[y][x] = desert_handle.clone();
                world_map[y][x] = TileType::Passable;
            }
        }
    }

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            if tile_map[y][x] == desert_handle {
                let value = perlin.get([x as f64 / 60.0, y as f64 / 60.0]);
                if value > 0.50 {
                    tile_map[y][x] = obstacle_handle.clone();
                    world_map[y][x] = TileType::Obstacle;
                }
            }
        }
    }

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            if tile_map[y][x] == desert_handle {
                let value = perlin.get([x as f64 / 16.0, y as f64 / 16.0]);
                if value > 0.60 {
                    tile_map[y][x] = science_handle.clone();
                    world_map[y][x] = TileType::Science;
                }
            }
        }
    }

    let offset_x = WIDTH as f32 * TILE_SIZE / 2.0;
    let offset_y = HEIGHT as f32 * TILE_SIZE / 2.0;

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let texture = tile_map[y][x].clone();
            let tile_type = world_map[y][x];
            
            commands.spawn((
                Tile {
                    tile_type,
                    x,
                    y,
                },
                SpriteBundle {
                    texture,
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                        ..Default::default()
                    },
                    transform: Transform::from_xyz(
                        x as f32 * TILE_SIZE - offset_x,
                        y as f32 * TILE_SIZE - offset_y,
                        0.0,
                    ),
                    ..Default::default()
                },
            ));
        }
    }

    let mut station_x = 0;
    let mut station_y = 0;
    'station_loop: for _ in 0..1000 {
        let x = rng.gen_range(5..(WIDTH - 5));
        let y = rng.gen_range(5..(HEIGHT - 5));

        let mut is_valid = true;
        for j in (y - 3)..=(y + 3) {
            for i in (x - 3)..=(x + 3) {
                if tile_map[j][i] != desert_handle {
                    is_valid = false;
                    break;
                }
            }
            if !is_valid {
                break;
            }
        }

        if is_valid {
            station_x = x;
            station_y = y;
            world_map[y][x] = TileType::Station;
            break 'station_loop;
        }
    }

    let world_x = station_x as f32 * TILE_SIZE - offset_x;
    let world_y = station_y as f32 * TILE_SIZE - offset_y;

    commands.spawn((
        Station,
        SpriteBundle {
            texture: station_handle.clone(),
            sprite: Sprite {
                custom_size: Some(Vec2::splat(TILE_SIZE * 12.0)),
                ..Default::default()
            },
            transform: Transform::from_xyz(world_x, world_y, 2.0),
            ..Default::default()
        },
    ));

    commands.insert_resource(StationPosition {
        x: station_x,
        y: station_y,
    });
    
    commands.insert_resource(WorldMap {
        tiles: world_map.clone(),
    });

    let directions = [(1, 0), (-1, 0)];
    for (_idx, (dx, dy)) in directions.iter().enumerate() {
        let rx = station_x as isize + dx;
        let ry = station_y as isize + dy;

        if rx >= 0 && rx < WIDTH as isize && ry >= 0 && ry < HEIGHT as isize {
            if world_map[ry as usize][rx as usize] == TileType::Passable {
                let mut memory = HashMap::new();
                for j in (station_y as isize - 10).max(0)..(station_y as isize + 10).min(HEIGHT as isize) {
                    for i in (station_x as isize - 10).max(0)..(station_x as isize + 10).min(WIDTH as isize) {
                        memory.insert((i as usize, j as usize), world_map[j as usize][i as usize]);
                    }
                }

                commands.spawn((
                    Robot {
                        x: rx as usize,
                        y: ry as usize,
                        current_goal: RobotGoal::Explore,
                        direction: Direction::random(),
                        directional_momentum: rng.gen_range(5..15),
                        exploration_timer: Timer::from_seconds(0.5, TimerMode::Repeating),
                        carrying_resource: None,
                    },
                    MapMemory {
                        tiles: memory,
                        known_resources: Vec::new(),
                    },
                    SpriteBundle {
                        texture: robot_handle.clone(),
                        sprite: Sprite {
                            custom_size: Some(Vec2::splat(TILE_SIZE * 6.0)),
                            ..Default::default()
                        },
                        transform: Transform::from_xyz(
                            rx as f32 * TILE_SIZE - offset_x,
                            ry as f32 * TILE_SIZE - offset_y,
                            2.0,
                        ),
                        ..Default::default()
                    },
                ));
            }
        }
    }

    let mut placed_positions: Vec<(usize, usize)> = Vec::new();

    let mut placed_minerals = 0;
    while placed_minerals < 150 {
        let candidate_x = rng.gen_range(RESOURCE_HALF_TILES..(WIDTH - RESOURCE_HALF_TILES));
        let candidate_y = rng.gen_range(RESOURCE_HALF_TILES..(HEIGHT - RESOURCE_HALF_TILES));

        let mut overlaps_obstacle = false;
        for j in (candidate_y - RESOURCE_HALF_TILES)..(candidate_y + RESOURCE_HALF_TILES) {
            for i in (candidate_x - RESOURCE_HALF_TILES)..(candidate_x + RESOURCE_HALF_TILES) {
                if world_map[j][i] == TileType::Obstacle {
                    overlaps_obstacle = true;
                    break;
                }
            }
            if overlaps_obstacle {
                break;
            }
        }
        if overlaps_obstacle {
            continue;
        }

        let mut overlaps_resource = false;
        for &(px, py) in &placed_positions {
            if (candidate_x as isize - px as isize).abs() < 10
                && (candidate_y as isize - py as isize).abs() < 10
            {
                overlaps_resource = true;
                break;
            }
        }
        if overlaps_resource {
            continue;
        }

        world_map[candidate_y][candidate_x] = TileType::Mineral;

        commands.spawn((
            ResourcePoint {
                resource_type: ResourceType::Mineral,
                x: candidate_x,
                y: candidate_y,
            },
            SpriteBundle {
                texture: mineral_handle.clone(),
                sprite: Sprite {
                    custom_size: Some(Vec2::new(MINERAL_SIZE, MINERAL_SIZE)),
                    ..Default::default()
                },
                transform: Transform::from_xyz(
                    candidate_x as f32 * TILE_SIZE - offset_x,
                    candidate_y as f32 * TILE_SIZE - offset_y,
                    1.0,
                ),
                ..Default::default()
            },
        ));
        placed_positions.push((candidate_x, candidate_y));
        placed_minerals += 1;
    }

    let mut placed_energy = 0;
    while placed_energy < 50 {
        let candidate_x = rng.gen_range(RESOURCE_HALF_TILES..(WIDTH - RESOURCE_HALF_TILES));
        let candidate_y = rng.gen_range(RESOURCE_HALF_TILES..(HEIGHT - RESOURCE_HALF_TILES));

        let mut overlaps_obstacle = false;
        for j in (candidate_y - RESOURCE_HALF_TILES)..(candidate_y + RESOURCE_HALF_TILES) {
            for i in (candidate_x - RESOURCE_HALF_TILES)..(candidate_x + RESOURCE_HALF_TILES) {
                if world_map[j][i] == TileType::Obstacle {
                    overlaps_obstacle = true;
                    break;
                }
            }
            if overlaps_obstacle {
                break;
            }
        }
        if overlaps_obstacle {
            continue;
        }

        let mut overlaps_resource = false;
        for &(px, py) in &placed_positions {
            if (candidate_x as isize - px as isize).abs() < 10
                && (candidate_y as isize - py as isize).abs() < 10
            {
                overlaps_resource = true;
                break;
            }
        }
        if overlaps_resource {
            continue;
        }

        world_map[candidate_y][candidate_x] = TileType::Energy;

        commands.spawn((
            ResourcePoint {
                resource_type: ResourceType::Energy,
                x: candidate_x,
                y: candidate_y,
            },
            SpriteBundle {
                texture: energy_handle.clone(),
                sprite: Sprite {
                    custom_size: Some(Vec2::new(MINERAL_SIZE, MINERAL_SIZE)),
                    ..Default::default()
                },
                transform: Transform::from_xyz(
                    candidate_x as f32 * TILE_SIZE - offset_x,
                    candidate_y as f32 * TILE_SIZE - offset_y,
                    1.0,
                ),
                ..Default::default()
            },
        ));
        placed_positions.push((candidate_x, candidate_y));
        placed_energy += 1;
    }
    
    println!("==== SETUP TERMINÉ ====");
    println!("Station placée en ({}, {})", station_x, station_y);
    println!("Minéraux placés: {}", placed_minerals);
    println!("Énergie placée: {}", placed_energy);
}

fn setup_ui(mut commands: Commands) {
    // Créer le texte pour les minéraux (en haut à gauche)
    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                "Minéraux: 0",
                TextStyle {
                    font_size: 30.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            transform: Transform::from_xyz(-550.0, 350.0, 100.0),
            ..default()
        },
        CounterText::Mineral,
    ));

    // Créer le texte pour l'énergie (en dessous)
    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                "Énergie: 0",
                TextStyle {
                    font_size: 30.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            transform: Transform::from_xyz(-550.0, 300.0, 100.0),
            ..default()
        },
        CounterText::Energy,
    ));
    
    println!("Interface utilisateur créée!");
}

// Mise à jour des textes des compteurs
fn update_counter_text(
    resource_counter: Res<ResourceCounter>,
    mut query: Query<(&mut Text, &CounterText)>,
) {
    for (mut text, counter_type) in query.iter_mut() {
        match counter_type {
            CounterText::Mineral => {
                text.sections[0].value = format!("Minéraux: {}", resource_counter.minerals);
            },
            CounterText::Energy => {
                text.sections[0].value = format!("Énergie: {}", resource_counter.energy);
            },
        }
    }
}

// Mise à jour de l'apparence des robots
fn update_robot_visuals(mut query: Query<(&Robot, &mut Sprite), Changed<Robot>>) {
    for (robot, mut sprite) in query.iter_mut() {
        if let Some(resource_type) = robot.carrying_resource {
            // Robot avec ressource: couleur différente
            match resource_type {
                ResourceType::Mineral => sprite.color = Color::rgba(1.0, 0.0, 0.0, 0.8), // Rouge pour minéraux
                ResourceType::Energy => sprite.color = Color::rgba(1.0, 1.0, 0.0, 0.8),  // Jaune pour énergie
            }
        } else {
            // Robot sans ressource: couleur normale
            sprite.color = Color::rgba(1.0, 1.0, 1.0, 1.0);
        }
    }
}

// NOUVEAU SYSTÈME SIMPLIFIÉ DE COLLECTE DES RESSOURCES
fn direct_resource_collection(
    mut commands: Commands,
    mut resource_counter: ResMut<ResourceCounter>,
    mut robots: Query<(&mut Robot, &Transform)>,
    resources: Query<(Entity, &ResourcePoint, &Transform)>,
    station_pos: Res<StationPosition>,
) {
    let offset_x = WIDTH as f32 * TILE_SIZE / 2.0;
    let offset_y = HEIGHT as f32 * TILE_SIZE / 2.0;
    
    // Pour chaque robot
    for (mut robot, robot_transform) in robots.iter_mut() {
        // Calculez la position actuelle du robot
        let rx = ((robot_transform.translation.x + offset_x) / TILE_SIZE).round() as usize;
        let ry = ((robot_transform.translation.y + offset_y) / TILE_SIZE).round() as usize;
        
        // Si le robot est près de la station et porte une ressource
        if let Some(resource_type) = robot.carrying_resource {
            if (rx as isize - station_pos.x as isize).abs() < 5 && 
               (ry as isize - station_pos.y as isize).abs() < 5 {
                // Incrémente le compteur
                match resource_type {
                    ResourceType::Mineral => {
                        resource_counter.minerals += 1;
                        println!("Minéraux: {}", resource_counter.minerals);
                    },
                    ResourceType::Energy => {
                        resource_counter.energy += 1;
                        println!("Énergie: {}", resource_counter.energy);
                    },
                }
                
                // Le robot ne porte plus de ressource
                robot.carrying_resource = None;
            }
        } 
        // Si le robot n'a pas de ressource, vérifie s'il peut en collecter une
        else {
            for (entity, resource, resource_transform) in resources.iter() {
                // Calculez la position de la ressource
                let resx = ((resource_transform.translation.x + offset_x) / TILE_SIZE).round() as usize;
                let resy = ((resource_transform.translation.y + offset_y) / TILE_SIZE).round() as usize;
                
                // Calcule la distance entre le robot et la ressource
                let distance = ((rx as f32 - resx as f32).powi(2) + (ry as f32 - resy as f32).powi(2)).sqrt();
                
                // Si le robot est assez proche (rayon de 10 unités)
                if distance < 10.0 {
                    // Le robot collecte la ressource
                    robot.carrying_resource = Some(resource.resource_type);
                    
                    // Supprime la ressource du monde
                    commands.entity(entity).despawn();
                    
                    println!("Robot collecte une ressource de type {:?}", resource.resource_type);
                    break; // Sortir de la boucle après avoir collecté une ressource
                }
            }
        }
    }
}

fn robot_observation_system(
    mut robots: Query<(&mut Robot, &mut MapMemory, &Transform)>,
    _tiles: Query<&Tile>,
    world_map: Res<WorldMap>,
    time: Res<Time>,
) {
    let offset_x = WIDTH as f32 * TILE_SIZE / 2.0;
    let offset_y = HEIGHT as f32 * TILE_SIZE / 2.0;
    
    for (mut robot, mut map_memory, transform) in robots.iter_mut() {
        robot.exploration_timer.tick(time.delta());
        
        let current_x = ((transform.translation.x + offset_x) / TILE_SIZE).round() as usize;
        let current_y = ((transform.translation.y + offset_y) / TILE_SIZE).round() as usize;
        
        robot.x = current_x.clamp(0, WIDTH - 1);
        robot.y = current_y.clamp(0, HEIGHT - 1);
        
        for dy in -(ROBOT_VIEW_DISTANCE as isize)..=(ROBOT_VIEW_DISTANCE as isize) {
            for dx in -(ROBOT_VIEW_DISTANCE as isize)..=(ROBOT_VIEW_DISTANCE as isize) {
                let obs_x = robot.x as isize + dx;
                let obs_y = robot.y as isize + dy;
                
                if obs_x >= 0 && obs_x < WIDTH as isize && obs_y >= 0 && obs_y < HEIGHT as isize {
                    let tile_type = world_map.tiles[obs_y as usize][obs_x as usize];
                    map_memory.tiles.insert((obs_x as usize, obs_y as usize), tile_type);
                    
                    if tile_type == TileType::Mineral || tile_type == TileType::Energy {
                        let resource_type = if tile_type == TileType::Mineral {
                            ResourceType::Mineral
                        } else {
                            ResourceType::Energy
                        };
                        
                        if !map_memory.known_resources.iter().any(|(x, y, _)| *x == obs_x as usize && *y == obs_y as usize) {
                            map_memory.known_resources.push((obs_x as usize, obs_y as usize, resource_type));
                        }
                    }
                }
            }
        }
    }
}

fn robot_decision_system(
    mut robots: Query<(&mut Robot, &MapMemory)>,
    station_pos: Res<StationPosition>,
    time: Res<Time>,
) {
    let mut rng = rand::thread_rng();
    
    for (mut robot, map_memory) in robots.iter_mut() {
        if !robot.exploration_timer.finished() {
            continue;
        }
        
        match robot.current_goal {
            RobotGoal::Explore => {
                // Si le robot transporte déjà une ressource, il retourne à la station
                if robot.carrying_resource.is_some() {
                    robot.current_goal = RobotGoal::ReturnToStation;
                } 
                // Sinon, il peut décider d'aller chercher une ressource ou continuer d'explorer
                else if !map_memory.known_resources.is_empty() && rng.gen_range(0..100) < 70 {
                    let resource_idx = rng.gen_range(0..map_memory.known_resources.len());
                    let (res_x, res_y, res_type) = map_memory.known_resources[resource_idx];
                    robot.current_goal = RobotGoal::GoToResource {
                        x: res_x,
                        y: res_y,
                        resource_type: res_type,
                    };
                } else {
                    // Continue d'explorer
                    robot.directional_momentum -= 1;
                    if robot.directional_momentum == 0 {
                        robot.direction = get_smart_exploration_direction(&robot, map_memory);
                        robot.directional_momentum = rng.gen_range(5..15);
                    }
                }
            },
            RobotGoal::GoToResource { .. } => {
                // Cette logique est maintenant gérée par direct_resource_collection
                // pour une meilleure réactivité
                if robot.carrying_resource.is_some() {
                    robot.current_goal = RobotGoal::ReturnToStation;
                }
            },
            RobotGoal::ReturnToStation => {
                // Si le robot est à la station, il dépose sa ressource et repart explorer
                if (robot.x as isize - station_pos.x as isize).abs() < 3 && 
                   (robot.y as isize - station_pos.y as isize).abs() < 3 {
                    robot.carrying_resource = None;
                    robot.current_goal = RobotGoal::Explore;
                    robot.direction = Direction::random();
                    robot.directional_momentum = rng.gen_range(5..15);
                }
            },
            RobotGoal::Idle => {
                if rng.gen_range(0..100) < 10 {
                    robot.current_goal = RobotGoal::Explore;
                    robot.direction = Direction::random();
                    robot.directional_momentum = rng.gen_range(5..15);
                }
            },
        }
    }
}

fn get_smart_exploration_direction(robot: &Robot, map_memory: &MapMemory) -> Direction {
    let mut rng = rand::thread_rng();
    let directions = [
        Direction::North,
        Direction::South,
        Direction::East,
        Direction::West,
        Direction::NorthEast,
        Direction::NorthWest,
        Direction::SouthEast,
        Direction::SouthWest,
    ];
    
    let mut direction_scores = vec![0; directions.len()];
    
    for (i, &dir) in directions.iter().enumerate() {
        let (dx, dy) = dir.get_direction_vector();
        
        let mut unexplored_count = 0;
        let mut obstacle_count = 0;
        
        for step in 1..10 {
            let check_x = (robot.x as isize + dx * step).clamp(0, WIDTH as isize - 1) as usize;
            let check_y = (robot.y as isize + dy * step).clamp(0, HEIGHT as isize - 1) as usize;
            
            if !map_memory.tiles.contains_key(&(check_x, check_y)) {
                unexplored_count += 1;
            } else if map_memory.tiles[&(check_x, check_y)] == TileType::Obstacle {
                obstacle_count += 1;
            }
        }
        
        direction_scores[i] = unexplored_count * 10 - obstacle_count * 5;
        direction_scores[i] += rng.gen_range(0..5);
    }
    
    let mut best_score = direction_scores[0];
    let mut best_idx = 0;
    
    for i in 1..direction_scores.len() {
        if direction_scores[i] > best_score {
            best_score = direction_scores[i];
            best_idx = i;
        }
    }
    
    if best_score <= 0 {
        return Direction::random();
    }
    
    directions[best_idx]
}

fn robot_movement_system(
    mut robots: Query<(&Robot, &MapMemory, &mut Transform)>,
    station_pos: Res<StationPosition>,
    time: Res<Time>,
) {
    let offset_x = WIDTH as f32 * TILE_SIZE / 2.0;
    let offset_y = HEIGHT as f32 * TILE_SIZE / 2.0;
    
    for (robot, map_memory, mut transform) in robots.iter_mut() {
        let mut dx = 0.0;
        let mut dy = 0.0;
        let speed = 10.0 * TILE_SIZE * time.delta_seconds();
        
        match robot.current_goal {
            RobotGoal::Explore => {
                let (dir_x, dir_y) = robot.direction.get_direction_vector();
                dx = dir_x as f32;
                dy = dir_y as f32;
                
                let next_x = (robot.x as isize + dir_x).clamp(0, WIDTH as isize - 1) as usize;
                let next_y = (robot.y as isize + dir_y).clamp(0, HEIGHT as isize - 1) as usize;
                
                if map_memory.tiles.contains_key(&(next_x, next_y)) && 
                   map_memory.tiles[&(next_x, next_y)] == TileType::Obstacle {
                    dx *= 0.1;
                    dy *= 0.1;
                }
            },
            RobotGoal::GoToResource { x, y, .. } => {
                let diff_x = x as isize - robot.x as isize;
                let diff_y = y as isize - robot.y as isize;
                
                if diff_x.abs() > 0 || diff_y.abs() > 0 {
                    let len = ((diff_x * diff_x + diff_y * diff_y) as f32).sqrt();
                    dx = diff_x as f32 / len;
                    dy = diff_y as f32 / len;
                }
                
                let next_x = (robot.x as isize + dx.signum() as isize).clamp(0, WIDTH as isize - 1) as usize;
                let next_y = (robot.y as isize + dy.signum() as isize).clamp(0, HEIGHT as isize - 1) as usize;
                
                if map_memory.tiles.contains_key(&(next_x, next_y)) && 
                   map_memory.tiles[&(next_x, next_y)] == TileType::Obstacle {
                    if dx.abs() > dy.abs() {
                        dy = if rand::random::<bool>() { 1.0 } else { -1.0 };
                        dx *= 0.2;
                    } else {
                        dx = if rand::random::<bool>() { 1.0 } else { -1.0 };
                        dy *= 0.2;
                    }
                }
            },
            RobotGoal::ReturnToStation => {
                let diff_x = station_pos.x as isize - robot.x as isize;
                let diff_y = station_pos.y as isize - robot.y as isize;
                
                if diff_x.abs() > 0 || diff_y.abs() > 0 {
                    let len = ((diff_x * diff_x + diff_y * diff_y) as f32).sqrt();
                    dx = diff_x as f32 / len;
                    dy = diff_y as f32 / len;
                }
                
                let next_x = (robot.x as isize + dx.signum() as isize).clamp(0, WIDTH as isize - 1) as usize;
                let next_y = (robot.y as isize + dy.signum() as isize).clamp(0, HEIGHT as isize - 1) as usize;
                
                if map_memory.tiles.contains_key(&(next_x, next_y)) && 
                   map_memory.tiles[&(next_x, next_y)] == TileType::Obstacle {
                    if dx.abs() > dy.abs() {
                        dy = if rand::random::<bool>() { 1.0 } else { -1.0 };
                        dx *= 0.2;
                    } else {
                        dx = if rand::random::<bool>() { 1.0 } else { -1.0 };
                        dy *= 0.2;
                    }
                }
            },
            RobotGoal::Idle => {
                dx = (rand::random::<f32>() - 0.5) * 0.5;
                dy = (rand::random::<f32>() - 0.5) * 0.5;
            },
        }
        
        transform.translation.x += dx * speed;
        transform.translation.y += dy * speed;
        
        let min_x = -offset_x + TILE_SIZE;
        let max_x = offset_x - TILE_SIZE;
        let min_y = -offset_y + TILE_SIZE;
        let max_y = offset_y - TILE_SIZE;
        
        transform.translation.x = transform.translation.x.clamp(min_x, max_x);
        transform.translation.y = transform.translation.y.clamp(min_y, max_y);
    }
}