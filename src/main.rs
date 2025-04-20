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

#[derive(Resource)]
struct SharedKnowledge {
    // Une valeur de 0 signifie "jamais exploré", une valeur élevée = souvent visité
    exploration_grid: Vec<Vec<u32>>,
    known_resources: Vec<(usize, usize, ResourceType)>,
    exploration_age: u32, // Compteur pour l'âge global de l'exploration
}

impl Default for SharedKnowledge {
    fn default() -> Self {
        // Initialiser la grille avec des zéros
        let grid = vec![vec![0; WIDTH]; HEIGHT];
        
        SharedKnowledge {
            exploration_grid: grid,
            known_resources: Vec::new(),
            exploration_age: 0,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ResourceType {
    Mineral,
    Energy,
}

#[derive(Component)]
struct Station;

#[derive(Resource, Default)]
struct GameTimer {
    elapsed_seconds: f32,
}

#[derive(Component)]
struct TimerText;

#[derive(Component)]
struct Robot {
    x: usize,
    y: usize,
    current_goal: RobotGoal,
    direction: Direction,
    directional_momentum: u32,
    exploration_timer: Timer,
    carrying_resource: Option<ResourceType>,
    // Nouveau champ pour détecter quand le robot est bloqué
    collision_timer: Option<f32>,
    last_position: (f32, f32),
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
        .insert_resource(GameTimer::default())
        .insert_resource(SharedKnowledge::default()) // Nouvelle ressource
        .add_systems(Startup, setup)
        .add_systems(Startup, setup_ui)
        .add_systems(Update, (
            robot_observation_system,
            share_knowledge_system, // Nouveau système de partage
            robot_decision_system,
            robot_movement_system,
            direct_resource_collection,
            update_robot_visuals,
            update_counter_text,
            update_timer,
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

    let directions = [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (-1, -1)]; // Ajouté 2 directions (haut et bas)
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
                        collision_timer: None,
                        last_position: (rx as f32 * TILE_SIZE - offset_x, ry as f32 * TILE_SIZE - offset_y),
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

fn update_timer(
    time: Res<Time>, 
    mut game_timer: ResMut<GameTimer>,
    mut query: Query<&mut Text, With<TimerText>>,
) {
    // Mettre à jour le temps écoulé
    game_timer.elapsed_seconds += time.delta_seconds();
    
    // Mettre à jour le texte du timer
    if let Ok(mut text) = query.get_single_mut() {
        let minutes = (game_timer.elapsed_seconds / 60.0) as u32;
        let seconds = (game_timer.elapsed_seconds % 60.0) as u32;
        text.sections[0].value = format!("Temps: {:02}:{:02}", minutes, seconds);
    }
}

fn setup_ui(mut commands: Commands) {
    // Texte pour les minéraux (en haut à gauche)
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

    // Texte pour l'énergie (en dessous)
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
    
    // NOUVEAU: Texte pour le timer (en dessous)
    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                "Temps: 00:00",
                TextStyle {
                    font_size: 30.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            transform: Transform::from_xyz(-550.0, 250.0, 100.0),
            ..default()
        },
        TimerText,
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

fn share_knowledge_system(
    robots: Query<(&Robot, &MapMemory, &Transform)>,
    mut shared_knowledge: ResMut<SharedKnowledge>,
) {
    let offset_x = WIDTH as f32 * TILE_SIZE / 2.0;
    let offset_y = HEIGHT as f32 * TILE_SIZE / 2.0;
    
    // Incrémenter l'âge global de l'exploration
    shared_knowledge.exploration_age += 1;
    
    // Si l'âge devient trop grand, on réduit l'intensité des visites partout
    if shared_knowledge.exploration_age > 100 {
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                if shared_knowledge.exploration_grid[y][x] > 0 {
                    // Les intensités sont divisées par 2 pour éviter que les anciennes zones restent "trop visitées"
                    shared_knowledge.exploration_grid[y][x] = shared_knowledge.exploration_grid[y][x] / 2;
                }
            }
        }
        shared_knowledge.exploration_age = 0;
    }
    
    // Pour chaque robot, marquer les zones explorées
    for (robot, map_memory, transform) in robots.iter() {
        let robot_x = ((transform.translation.x + offset_x) / TILE_SIZE).round() as usize;
        let robot_y = ((transform.translation.y + offset_y) / TILE_SIZE).round() as usize;
        
        // Marquer la position actuelle du robot et ses environs
        for dy in -3..=3 {
            for dx in -3..=3 {
                let x = (robot_x as isize + dx).clamp(0, WIDTH as isize - 1) as usize;
                let y = (robot_y as isize + dy).clamp(0, HEIGHT as isize - 1) as usize;
                
                // Distance au robot
                let distance = ((dx * dx + dy * dy) as f32).sqrt();
                
                // Plus c'est proche du robot, plus le marquage est intense
                if distance <= 3.0 {
                    shared_knowledge.exploration_grid[y][x] = (shared_knowledge.exploration_grid[y][x] + 1).min(200);
                }
            }
        }
        
        // Ajouter toutes les cases observées mais moins intensément
        for (&(tile_x, tile_y), &tile_type) in &map_memory.tiles {
            if tile_x < WIDTH && tile_y < HEIGHT {
                // Marquer comme "observé" plutôt que "visité"
                shared_knowledge.exploration_grid[tile_y][tile_x] = 
                    shared_knowledge.exploration_grid[tile_y][tile_x].saturating_add(1).min(10);
            }
        }
    }
    
    // Partager les informations sur les ressources entre tous les robots
    let mut all_resources = Vec::new();
    for (_, map_memory, _) in robots.iter() {
        for &resource_info in &map_memory.known_resources {
            if !all_resources.contains(&resource_info) {
                all_resources.push(resource_info);
            }
        }
    }
    shared_knowledge.known_resources = all_resources;
}

fn find_exploration_direction(robot: &Robot, map_memory: &MapMemory, exploration_grid: &Vec<Vec<u32>>) -> Direction {
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
    
    let mut direction_scores = vec![0.0; directions.len()];
    
    for (i, &dir) in directions.iter().enumerate() {
        let (dx, dy) = dir.get_direction_vector();
        
        let mut exploration_score = 0.0;
        let mut obstacle_penalty = 0.0;
        let mut border_penalty = 0.0;
        
        // Examiner les cases dans cette direction (jusqu'à 25 cases)
        for step in 1..=25 {
            let x = (robot.x as isize + dx * step).clamp(0, WIDTH as isize - 1) as usize;
            let y = (robot.y as isize + dy * step).clamp(0, HEIGHT as isize - 1) as usize;
            
            // Vérifier si on est près du bord
            if x <= 5 || x >= WIDTH - 5 || y <= 5 || y >= HEIGHT - 5 {
                border_penalty += 5.0 / step as f32; // Pénalité moins forte pour les bords lointains
            }
            
            // Vérifier si on a un obstacle
            if map_memory.tiles.get(&(x, y)).map_or(false, |&t| t == TileType::Obstacle) {
                obstacle_penalty += 10.0 / step as f32; // Pénalité moins forte pour les obstacles lointains
            }
            
            // Valeur d'exploration - bonus pour les cases peu explorées
            let exploration_value = if exploration_grid[y][x] == 0 {
                // Bonus pour l'inexploré
                10.0 / step as f32
            } else {
                // Moins l'intensité d'exploration est forte, plus le score est élevé
                5.0 / (exploration_grid[y][x] as f32 * step as f32)
            };
            
            exploration_score += exploration_value;
        }
        
        // Score final
        direction_scores[i] = exploration_score - obstacle_penalty - border_penalty;
        
        // Ajout d'aléatoire pour éviter les comportements trop prévisibles
        direction_scores[i] += rng.gen_range(0.0..3.0);
    }
    
    // Trouver les directions avec les meilleurs scores
    let mut best_directions = Vec::new();
    let mut best_score = f32::MIN;
    
    for (i, score) in direction_scores.iter().enumerate() {
        if *score > best_score {
            best_score = *score;
            best_directions.clear();
            best_directions.push(i);
        } else if (*score - best_score).abs() < 0.1 {
            // Considérer les scores très proches comme équivalents
            best_directions.push(i);
        }
    }
    
    // Si on a plusieurs directions équivalentes, en choisir une au hasard
    if !best_directions.is_empty() {
        let idx = if best_directions.len() > 1 {
            best_directions[rng.gen_range(0..best_directions.len())]
        } else {
            best_directions[0]
        };
        directions[idx]
    } else {
        // Cas de secours - choisir une direction aléatoire
        Direction::random()
    }
}

fn find_path(start: (usize, usize), goal: (usize, usize), map_memory: &MapMemory) -> Vec<(usize, usize)> {
    use std::collections::BinaryHeap;
    use std::cmp::Ordering;
    
    #[derive(Copy, Clone, Eq, PartialEq)]
    struct Node {
        position: (usize, usize),
        f_score: u32,
    }
    
    impl Ord for Node {
        fn cmp(&self, other: &Self) -> Ordering {
            // Inversion pour avoir un min-heap
            other.f_score.cmp(&self.f_score)
        }
    }
    
    impl PartialOrd for Node {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }
    
    // Si le but est très proche, renvoyer directement la position du but
    let dist_to_goal = (start.0.abs_diff(goal.0) + start.1.abs_diff(goal.1)) as usize;
    if dist_to_goal <= 1 {
        return vec![goal];
    }
    
    // Fonction heuristique (distance de Manhattan)
    fn heuristic(a: (usize, usize), b: (usize, usize)) -> u32 {
        (a.0.abs_diff(b.0) + a.1.abs_diff(b.1)) as u32
    }
    
    let mut open_set = BinaryHeap::new();
    let mut came_from = HashMap::new();
    let mut g_score = HashMap::new();
    
    // Initialisation
    g_score.insert(start, 0);
    open_set.push(Node {
        position: start,
        f_score: heuristic(start, goal),
    });
    
    // Directions possibles (8 directions)
    let directions = [
        (0, 1), (1, 0), (0, -1), (-1, 0),  // Cardinales
        (1, 1), (1, -1), (-1, 1), (-1, -1)  // Diagonales
    ];
    
    // Limiter le nombre d'itérations pour éviter des calculs trop longs
    let max_iterations = 1000;
    let mut iterations = 0;
    
    while let Some(current) = open_set.pop() {
        iterations += 1;
        if iterations > max_iterations {
            // Si on dépasse le nombre max d'itérations, retourner le meilleur chemin trouvé jusqu'ici
            if came_from.contains_key(&goal) {
                let mut path = Vec::new();
                let mut current_pos = goal;
                
                while current_pos != start {
                    path.push(current_pos);
                    current_pos = came_from[&current_pos];
                }
                
                path.reverse();
                return path;
            }
            break;
        }
        
        let (x, y) = current.position;
        
        // Vérifier si on est assez proche du but
        if (x as isize - goal.0 as isize).abs() < 2 && (y as isize - goal.1 as isize).abs() < 2 {
            // Reconstruire le chemin
            let mut path = Vec::new();
            let mut current_pos = (x, y);
            
            // Ajouter le but exact au chemin
            path.push(goal);
            
            while current_pos != start {
                if !came_from.contains_key(&current_pos) {
                    break;
                }
                path.push(current_pos);
                current_pos = came_from[&current_pos];
            }
            
            // Inverser le chemin pour avoir le début en premier
            path.reverse();
            
            // Ne retourner que les premières étapes pour optimiser
            if path.len() > 3 {
                return path[0..3].to_vec();
            }
            return path;
        }
        
        let current_g = *g_score.get(&(x, y)).unwrap_or(&u32::MAX);
        
        // Explorer les voisins
        for &(dx, dy) in &directions {
            let nx = (x as isize + dx).clamp(0, WIDTH as isize - 1) as usize;
            let ny = (y as isize + dy).clamp(0, HEIGHT as isize - 1) as usize;
            let neighbor = (nx, ny);
            
            // Vérifier si le voisin est un obstacle
            let is_obstacle = map_memory.tiles.get(&neighbor)
                .map(|&tile_type| tile_type == TileType::Obstacle)
                .unwrap_or(false);
            
            if is_obstacle {
                continue; // Ignorer les obstacles
            }
            
            // Coût pour se déplacer vers ce voisin
            let move_cost = if dx.abs() + dy.abs() > 1 { 14 } else { 10 }; // Coût diagonal = 1.4, cardinal = 1
            let tentative_g = current_g + move_cost;
            
            // Si on a trouvé un meilleur chemin
            if tentative_g < *g_score.get(&neighbor).unwrap_or(&u32::MAX) {
                came_from.insert(neighbor, (x, y));
                g_score.insert(neighbor, tentative_g);
                
                let f_score = tentative_g + heuristic(neighbor, goal);
                open_set.push(Node {
                    position: neighbor,
                    f_score,
                });
            }
        }
    }
    
    // Si aucun chemin n'est trouvé, renvoyer une direction approximative vers le but
    let dx = if goal.0 > start.0 { 1 } else if goal.0 < start.0 { -1 } else { 0 };
    let dy = if goal.1 > start.1 { 1 } else if goal.1 < start.1 { -1 } else { 0 };
    
    let next_x = (start.0 as isize + dx).clamp(0, WIDTH as isize - 1) as usize;
    let next_y = (start.1 as isize + dy).clamp(0, HEIGHT as isize - 1) as usize;
    
    // Vérifier si cette direction est un obstacle
    let is_obstacle = map_memory.tiles.get(&(next_x, next_y))
        .map(|&tile_type| tile_type == TileType::Obstacle)
        .unwrap_or(false);
    
    if !is_obstacle {
        vec![(next_x, next_y)]
    } else {
        // Chercher une direction libre
        for &(dx, dy) in &directions {
            let nx = (start.0 as isize + dx).clamp(0, WIDTH as isize - 1) as usize;
            let ny = (start.1 as isize + dy).clamp(0, HEIGHT as isize - 1) as usize;
            
            let is_obstacle = map_memory.tiles.get(&(nx, ny))
                .map(|&tile_type| tile_type == TileType::Obstacle)
                .unwrap_or(false);
            
            if !is_obstacle {
                return vec![(nx, ny)];
            }
        }
        
        // Si on ne trouve aucune direction libre, renvoyer vide
        Vec::new()
    }
}

// NOUVEAU SYSTÈME SIMPLIFIÉ DE COLLECTE DES RESSOURCES
fn direct_resource_collection(
    mut commands: Commands,
    mut resource_counter: ResMut<ResourceCounter>,
    mut robots: Query<(&mut Robot, &Transform)>,
    resources: Query<(Entity, &ResourcePoint, &Transform)>,
    mut shared_knowledge: ResMut<SharedKnowledge>,
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
                    
                    // Supprime la ressource de la connaissance partagée
                    shared_knowledge.known_resources.retain(|&(x, y, _)| x != resx || y != resy);
                    
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
        
        // Amélioration : garder une trace des ressources collectées
        let mut resources_to_remove = Vec::new();
        for (i, &(res_x, res_y, _)) in map_memory.known_resources.iter().enumerate() {
            if world_map.tiles[res_y][res_x] != TileType::Mineral && 
               world_map.tiles[res_y][res_x] != TileType::Energy {
                resources_to_remove.push(i);
            }
        }
        
        // Supprimer les ressources qui ne sont plus présentes (en commençant par la fin)
        for &i in resources_to_remove.iter().rev() {
            map_memory.known_resources.remove(i);
        }
        
        // Observation avec un rayon de vision augmenté
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
    shared_knowledge: Res<SharedKnowledge>,
    _time: Res<Time>,
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
                else {
                    // Dans tous les cas, on vérifie d'abord si on a des ressources connues
                    let should_seek_resource = !shared_knowledge.known_resources.is_empty() && rng.gen_range(0..100) < 70;
                    
                    if should_seek_resource {
                        // Trouver la meilleure ressource à cibler
                        let mut best_resource = None;
                        let mut best_score = f32::MIN;
                        
                        for &(res_x, res_y, res_type) in &shared_knowledge.known_resources {
                            // Calculer un score basé sur la distance et l'exploration
                            let distance = ((robot.x as f32 - res_x as f32).powi(2) + 
                                          (robot.y as f32 - res_y as f32).powi(2)).sqrt();
                            
                            // Score = 1000/(distance+1) - exploration*0.5
                            let score = 1000.0 / (distance + 1.0) - 
                                      shared_knowledge.exploration_grid[res_y][res_x] as f32 * 0.5;
                            
                            if score > best_score {
                                best_score = score;
                                best_resource = Some((res_x, res_y, res_type));
                            }
                        }
                        
                        // Si on a trouvé une ressource, la cibler
                        if let Some((x, y, res_type)) = best_resource {
                            robot.current_goal = RobotGoal::GoToResource {
                                x,
                                y,
                                resource_type: res_type,
                            };
                            println!("Robot décide d'aller chercher une ressource à ({}, {})", x, y);
                        } else {
                            // Sinon, continuer l'exploration
                            robot.directional_momentum -= 1;
                            
                            if robot.directional_momentum == 0 {
                                robot.direction = find_exploration_direction(&robot, map_memory, &shared_knowledge.exploration_grid);
                                robot.directional_momentum = rng.gen_range(10..20);
                            }
                        }
                    } else {
                        // Continuer l'exploration
                        robot.directional_momentum -= 1;
                        
                        if robot.directional_momentum == 0 {
                            robot.direction = find_exploration_direction(&robot, map_memory, &shared_knowledge.exploration_grid);
                            robot.directional_momentum = rng.gen_range(10..20);
                        }
                    }
                }
            },
            RobotGoal::GoToResource { x, y, resource_type } => {
                // Vérifier si la ressource existe encore
                let resource_exists = shared_knowledge.known_resources.contains(&(x, y, resource_type));
                
                if !resource_exists && robot.carrying_resource.is_none() {
                    // La ressource n'existe plus, retourner à l'exploration
                    robot.current_goal = RobotGoal::Explore;
                    robot.direction = find_exploration_direction(&robot, map_memory, &shared_knowledge.exploration_grid);
                    robot.directional_momentum = rng.gen_range(10..20);
                }
                // Si le robot a déjà une ressource, retourner à la station
                else if robot.carrying_resource.is_some() {
                    robot.current_goal = RobotGoal::ReturnToStation;
                }
            },
            RobotGoal::ReturnToStation => {
                // Si le robot est à la station, il dépose sa ressource et repart explorer
                if (robot.x as isize - station_pos.x as isize).abs() < 3 && 
                   (robot.y as isize - station_pos.y as isize).abs() < 3 {
                    robot.carrying_resource = None;
                    robot.current_goal = RobotGoal::Explore;
                    
                    // Quand un robot revient à la station, lui donner une direction qui l'éloigne de la station
                    // en priorité dans les zones les moins explorées
                    robot.direction = find_exploration_direction(&robot, map_memory, &shared_knowledge.exploration_grid);
                    // Momentum plus élevé pour s'éloigner de la station
                    robot.directional_momentum = rng.gen_range(15..30);
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

fn get_exploration_direction(robot: &Robot, map_memory: &MapMemory, heatmap: &HashMap<(usize, usize), u32>) -> Direction {
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
    
    let mut direction_scores = vec![0.0; directions.len()];
    
    for (i, &dir) in directions.iter().enumerate() {
        let (dx, dy) = dir.get_direction_vector();
        
        let mut unexplored_count = 0;
        let mut obstacle_count = 0;
        let mut heat_score = 0.0;
        let mut border_penalty = 0;
        
        // Vérifier plus loin (jusqu'à 20 cases)
        for step in 1..20 {
            let check_x = (robot.x as isize + dx * step).clamp(0, WIDTH as isize - 1) as usize;
            let check_y = (robot.y as isize + dy * step).clamp(0, HEIGHT as isize - 1) as usize;
            
            // Pénalité pour les bords de la carte
            if check_x <= 5 || check_x >= WIDTH - 5 || check_y <= 5 || check_y >= HEIGHT - 5 {
                border_penalty += 5;
            }
            
            // Vérifier si la case est explorée selon la carte de chaleur
            let visit_count = heatmap.get(&(check_x, check_y)).cloned().unwrap_or(0);
            
            // Plus la case a été visitée, moins elle est intéressante
            heat_score -= visit_count as f32 * (20.0 / (step as f32 + 1.0));
            
            if !map_memory.tiles.contains_key(&(check_x, check_y)) {
                unexplored_count += 1;
            } else if map_memory.tiles[&(check_x, check_y)] == TileType::Obstacle {
                obstacle_count += 1;
            }
        }
        
        // Calcul du score
        direction_scores[i] = 
            unexplored_count as f32 * 20.0 +    // Récompense élevée pour l'inexploré
            heat_score -                        // Pénalité pour les zones visitées
            obstacle_count as f32 * 15.0 -      // Pénalité pour les obstacles
            border_penalty as f32;              // Pénalité pour les bords
            
        // Ajouter un élément aléatoire
        direction_scores[i] += rng.gen_range(-5.0..5.0);
    }
    
    // Trouver la meilleure direction
    let mut best_idx = 0;
    let mut best_score = direction_scores[0];
    
    for i in 1..direction_scores.len() {
        if direction_scores[i] > best_score {
            best_score = direction_scores[i];
            best_idx = i;
        }
    }
    
    // Si tous les scores sont mauvais, choisir une direction complètement aléatoire
    // avec une préférence pour les directions diagonales (qui font aller plus loin)
    if best_score <= 0.0 {
        let diagonal_directions = [
            Direction::NorthEast,
            Direction::NorthWest,
            Direction::SouthEast,
            Direction::SouthWest,
        ];
        
        if rng.gen_bool(0.7) { // 70% de chance de choisir une direction diagonale
            return diagonal_directions[rng.gen_range(0..diagonal_directions.len())];
        } else {
            return Direction::random();
        }
    }
    
    directions[best_idx]
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
        let mut passable_count = 0;
        let mut is_border = false;
        
        for step in 1..15 { // Augmenté à 15 pour voir plus loin
            let check_x = (robot.x as isize + dx * step).clamp(0, WIDTH as isize - 1) as usize;
            let check_y = (robot.y as isize + dy * step).clamp(0, HEIGHT as isize - 1) as usize;
            
            // Vérifier si on atteint le bord de la carte
            if check_x == 0 || check_x == WIDTH - 1 || check_y == 0 || check_y == HEIGHT - 1 {
                is_border = true;
            }
            
            if !map_memory.tiles.contains_key(&(check_x, check_y)) {
                unexplored_count += 1;
            } else {
                match map_memory.tiles[&(check_x, check_y)] {
                    TileType::Obstacle => obstacle_count += 1,
                    TileType::Passable => passable_count += 1,
                    TileType::Mineral | TileType::Energy => passable_count += 2, // Bonus pour les ressources
                    _ => {}
                }
            }
        }
        
        // Calcul du score avec des poids améliorés
        direction_scores[i] = unexplored_count * 15  // Favoriser l'exploration (augmenté)
                             + passable_count * 5    // Favoriser les chemins dégagés (nouveau)
                             - obstacle_count * 10   // Éviter les obstacles (augmenté)
                             - if is_border { 20 } else { 0 }; // Pénalité pour les bords (nouveau)
                             
        // Ajouter une petite randomisation pour éviter les comportements répétitifs
        direction_scores[i] += rng.gen_range(0..8);
    }
    
    // Trouver la meilleure direction
    let mut best_score = direction_scores[0];
    let mut best_idx = 0;
    
    for i in 1..direction_scores.len() {
        if direction_scores[i] > best_score {
            best_score = direction_scores[i];
            best_idx = i;
        }
    }
    
    // Si tous les scores sont mauvais, choisir une direction complètement aléatoire
    if best_score <= 5 {
        return Direction::random();
    }
    
    directions[best_idx]
}

fn robot_movement_system(
    mut robots: Query<(&mut Robot, &MapMemory, &mut Transform)>,
    station_pos: Res<StationPosition>,
    shared_knowledge: Res<SharedKnowledge>,
    time: Res<Time>,
) {
    let offset_x = WIDTH as f32 * TILE_SIZE / 2.0;
    let offset_y = HEIGHT as f32 * TILE_SIZE / 2.0;
    
    for (mut robot, map_memory, mut transform) in robots.iter_mut() {
        let mut dx = 0.0;
        let mut dy = 0.0;
        let speed = 10.0 * TILE_SIZE * time.delta_seconds();
        
        // Position actuelle du robot
        let robot_x = ((transform.translation.x + offset_x) / TILE_SIZE).round() as usize;
        let robot_y = ((transform.translation.y + offset_y) / TILE_SIZE).round() as usize;
        
        robot.x = robot_x;
        robot.y = robot_y;
        
        // Détecter si le robot est bloqué (n'a pas bougé significativement)
        let current_pos = (transform.translation.x, transform.translation.y);
        let distance_moved = ((current_pos.0 - robot.last_position.0).powi(2) + 
                             (current_pos.1 - robot.last_position.1).powi(2)).sqrt();
        
        // Si le robot a à peine bougé, incrémenter le timer de collision
        if distance_moved < 0.5 * TILE_SIZE {
            robot.collision_timer = Some(robot.collision_timer.unwrap_or(0.0) + time.delta_seconds());
            
            // Si le robot est bloqué depuis plus de 0.2 secondes, changer immédiatement de direction
            if robot.collision_timer.unwrap_or(0.0) > 0.2 {
                match robot.current_goal {
                    RobotGoal::Explore => {
                        // Changer complètement de direction
                        robot.direction = get_opposite_direction(robot.direction);
                        robot.directional_momentum = 5; // Courte durée pour tester rapidement une autre direction
                    },
                    RobotGoal::GoToResource { .. } | RobotGoal::ReturnToStation => {
                        // Pour les objectifs précis, recalculer un nouveau chemin
                        // Pas besoin de changer l'objectif, juste de forcer un nouveau calcul de chemin
                    },
                    _ => {}
                }
                
                // Réinitialiser le timer de collision
                robot.collision_timer = None;
            }
        } else {
            // Le robot a bougé, réinitialiser le timer de collision
            robot.collision_timer = None;
        }
        
        // Mettre à jour la dernière position connue
        robot.last_position = current_pos;
        
        // Le reste du code de mouvement reste similaire
        match robot.current_goal {
            RobotGoal::Explore => {
                let (dir_x, dir_y) = robot.direction.get_direction_vector();
                
                // Vérifier d'abord si un obstacle est immédiatement devant
                let next_x = (robot_x as isize + dir_x).clamp(0, WIDTH as isize - 1) as usize;
                let next_y = (robot_y as isize + dir_y).clamp(0, HEIGHT as isize - 1) as usize;
                
                let is_obstacle = map_memory.tiles.get(&(next_x, next_y))
                    .map(|&tile_type| tile_type == TileType::Obstacle)
                    .unwrap_or(false);
                
                if is_obstacle {
                    // Obstacle détecté, changer immédiatement de direction
                    let alternatives = [
                        (dir_y, -dir_x),   // 90° à droite
                        (-dir_y, dir_x),   // 90° à gauche
                        (-dir_x, -dir_y),  // 180°
                    ];
                    
                    // Tester toutes les directions alternatives
                    let mut found_alternative = false;
                    for &(alt_dx, alt_dy) in &alternatives {
                        let alt_x = (robot_x as isize + alt_dx).clamp(0, WIDTH as isize - 1) as usize;
                        let alt_y = (robot_y as isize + alt_dy).clamp(0, HEIGHT as isize - 1) as usize;
                        
                        let is_obstacle = map_memory.tiles.get(&(alt_x, alt_y))
                            .map(|&tile_type| tile_type == TileType::Obstacle)
                            .unwrap_or(false);
                        
                        if !is_obstacle {
                            dx = alt_dx as f32;
                            dy = alt_dy as f32;
                            found_alternative = true;
                            break;
                        }
                    }
                    
                    // Si aucune direction n'est libre, essayer une direction aléatoire
                    if !found_alternative {
                        let random_dir = Direction::random();
                        let (rand_dx, rand_dy) = random_dir.get_direction_vector();
                        dx = rand_dx as f32;
                        dy = rand_dy as f32;
                        
                        // Mettre à jour la direction du robot
                        robot.direction = random_dir;
                    }
                } else {
                    // Pas d'obstacle immédiat, continuer dans la direction actuelle
                    dx = dir_x as f32;
                    dy = dir_y as f32;
                }
            },
            RobotGoal::GoToResource { x, y, .. } => {
                // Utiliser une approche plus directe vers la ressource
                let path = find_path((robot_x, robot_y), (x, y), map_memory);
                
                if let Some(&next_pos) = path.first() {
                    dx = next_pos.0 as f32 - robot_x as f32;
                    dy = next_pos.1 as f32 - robot_y as f32;
                } else {
                    // Pas de chemin possible, essayer l'approche directe
                    let diff_x = x as isize - robot_x as isize;
                    let diff_y = y as isize - robot_y as isize;
                    
                    if diff_x.abs() > 0 || diff_y.abs() > 0 {
                        // Normaliser le vecteur
                        let len = ((diff_x * diff_x + diff_y * diff_y) as f32).sqrt();
                        dx = diff_x as f32 / len;
                        dy = diff_y as f32 / len;
                        
                        // Vérifier si la direction est bloquée
                        let next_x = (robot_x as isize + dx.signum() as isize).clamp(0, WIDTH as isize - 1) as usize;
                        let next_y = (robot_y as isize + dy.signum() as isize).clamp(0, HEIGHT as isize - 1) as usize;
                        
                        let is_obstacle = map_memory.tiles.get(&(next_x, next_y))
                            .map(|&tile_type| tile_type == TileType::Obstacle)
                            .unwrap_or(false);
                            
                        if is_obstacle {
                            // Essayer toutes les directions possibles
                            let directions = [
                                (1, 0), (-1, 0), (0, 1), (0, -1),
                                (1, 1), (1, -1), (-1, 1), (-1, -1)
                            ];
                            
                            let mut best_dir = (0, 0);
                            let mut best_score = f32::MAX;
                            
                            for &(dir_x, dir_y) in &directions {
                                let alt_x = (robot_x as isize + dir_x).clamp(0, WIDTH as isize - 1) as usize;
                                let alt_y = (robot_y as isize + dir_y).clamp(0, HEIGHT as isize - 1) as usize;
                                
                                let is_obstacle = map_memory.tiles.get(&(alt_x, alt_y))
                                    .map(|&tile_type| tile_type == TileType::Obstacle)
                                    .unwrap_or(false);
                                
                                if !is_obstacle {
                                    // Calculer un score basé sur la distance au but
                                    let dist_to_goal = ((alt_x as isize - x as isize).pow(2) + 
                                                       (alt_y as isize - y as isize).pow(2)) as f32;
                                    
                                    if dist_to_goal < best_score {
                                        best_score = dist_to_goal;
                                        best_dir = (dir_x, dir_y);
                                    }
                                }
                            }
                            
                            // Utiliser la meilleure direction alternative
                            if best_dir != (0, 0) {
                                dx = best_dir.0 as f32;
                                dy = best_dir.1 as f32;
                            } else {
                                // Si aucune direction n'est libre, rester sur place
                                dx = 0.0;
                                dy = 0.0;
                            }
                        }
                    }
                }
            },
            RobotGoal::ReturnToStation => {
                // Approche similaire à GoToResource
                let path = find_path((robot_x, robot_y), (station_pos.x, station_pos.y), map_memory);
                
                if let Some(&next_pos) = path.first() {
                    dx = next_pos.0 as f32 - robot_x as f32;
                    dy = next_pos.1 as f32 - robot_y as f32;
                } else {
                    let diff_x = station_pos.x as isize - robot_x as isize;
                    let diff_y = station_pos.y as isize - robot_y as isize;
                    
                    if diff_x.abs() > 0 || diff_y.abs() > 0 {
                        let len = ((diff_x * diff_x + diff_y * diff_y) as f32).sqrt();
                        dx = diff_x as f32 / len;
                        dy = diff_y as f32 / len;
                        
                        // Vérification d'obstacle comme pour GoToResource
                        let next_x = (robot_x as isize + dx.signum() as isize).clamp(0, WIDTH as isize - 1) as usize;
                        let next_y = (robot_y as isize + dy.signum() as isize).clamp(0, HEIGHT as isize - 1) as usize;
                        
                        let is_obstacle = map_memory.tiles.get(&(next_x, next_y))
                            .map(|&tile_type| tile_type == TileType::Obstacle)
                            .unwrap_or(false);
                            
                        if is_obstacle {
                            // Logique identique à celle de GoToResource
                            let directions = [
                                (1, 0), (-1, 0), (0, 1), (0, -1),
                                (1, 1), (1, -1), (-1, 1), (-1, -1)
                            ];
                            
                            let mut best_dir = (0, 0);
                            let mut best_score = f32::MAX;
                            
                            for &(dir_x, dir_y) in &directions {
                                let alt_x = (robot_x as isize + dir_x).clamp(0, WIDTH as isize - 1) as usize;
                                let alt_y = (robot_y as isize + dir_y).clamp(0, HEIGHT as isize - 1) as usize;
                                
                                let is_obstacle = map_memory.tiles.get(&(alt_x, alt_y))
                                    .map(|&tile_type| tile_type == TileType::Obstacle)
                                    .unwrap_or(false);
                                
                                if !is_obstacle {
                                    let dist_to_station = ((alt_x as isize - station_pos.x as isize).pow(2) + 
                                                         (alt_y as isize - station_pos.y as isize).pow(2)) as f32;
                                    
                                    if dist_to_station < best_score {
                                        best_score = dist_to_station;
                                        best_dir = (dir_x, dir_y);
                                    }
                                }
                            }
                            
                            if best_dir != (0, 0) {
                                dx = best_dir.0 as f32;
                                dy = best_dir.1 as f32;
                            } else {
                                dx = 0.0;
                                dy = 0.0;
                            }
                        }
                    }
                }
            },
            RobotGoal::Idle => {
                dx = (rand::random::<f32>() - 0.5) * 0.5;
                dy = (rand::random::<f32>() - 0.5) * 0.5;
            },
        }
        
        // Normaliser le vecteur pour une vitesse constante
        let magnitude = (dx * dx + dy * dy).sqrt();
        if magnitude > 0.0 {
            dx /= magnitude;
            dy /= magnitude;
        }
        
        // Déplacer le robot
        transform.translation.x += dx * speed;
        transform.translation.y += dy * speed;
        
        // Limiter aux bords de la carte
        let min_x = -offset_x + TILE_SIZE;
        let max_x = offset_x - TILE_SIZE;
        let min_y = -offset_y + TILE_SIZE;
        let max_y = offset_y - TILE_SIZE;
        
        transform.translation.x = transform.translation.x.clamp(min_x, max_x);
        transform.translation.y = transform.translation.y.clamp(min_y, max_y);
    }
}

fn get_opposite_direction(dir: Direction) -> Direction {
    match dir {
        Direction::North => Direction::South,
        Direction::South => Direction::North,
        Direction::East => Direction::West,
        Direction::West => Direction::East,
        Direction::NorthEast => Direction::SouthWest,
        Direction::NorthWest => Direction::SouthEast,
        Direction::SouthEast => Direction::NorthWest,
        Direction::SouthWest => Direction::NorthEast,
    }
}