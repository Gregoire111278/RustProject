use bevy::prelude::*;
use noise::{NoiseFn, Perlin, Seedable};
use rand::Rng;

const TILE_SIZE: f32 = 4.0;
const WIDTH: usize = 370;  // Nouvelle largeur de la carte
const HEIGHT: usize = 190; // Nouvelle hauteur de la carte
const MINERAL_SIZE: f32 = TILE_SIZE * 10.0; // Taille du sprite en pixels (40.0)
                                           // Correspond à 10 tuiles (40/4)
const RESOURCE_HALF_TILES: usize = (MINERAL_SIZE / TILE_SIZE / 2.0) as usize; // 10/2 = 5

#[derive(Component)]
struct Tile;

#[derive(Component)]
struct ResourcePoint;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 1000.0),
        ..Default::default()
    });

    let seed = 14;
    let perlin = Perlin::new().set_seed(seed);
    let mut rng = rand::thread_rng();

    let desert_handle = asset_server.load("desert.jpg");
    let obstacle_handle = asset_server.load("obstacle.jpg");
    let science_handle = asset_server.load("gris.jpg");
    let mineral_handle = asset_server.load("rubis.png");
    let energy_handle = asset_server.load("eclair.png");

    let mut tile_map: Vec<Vec<Handle<Image>>> =
        vec![vec![desert_handle.clone(); WIDTH]; HEIGHT];

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let value = perlin.get([x as f64 / 5.0, y as f64 / 5.0]);
            if value < 0.50 {
                tile_map[y][x] = desert_handle.clone();
            }
        }
    }

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            if tile_map[y][x] == desert_handle {
                let value = perlin.get([x as f64 / 60.0, y as f64 / 60.0]);
                if value > 0.50 {
                    tile_map[y][x] = obstacle_handle.clone();
                }
            }
        }
    }

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            if tile_map[y][x] == desert_handle {
                let value = perlin.get([x as f64 / 8.0, y as f64 / 8.0]);
                if value > 0.60 {
                    tile_map[y][x] = science_handle.clone();
                }
            }
        }
    }

    let offset_x = WIDTH as f32 * TILE_SIZE / 2.0;
    let offset_y = HEIGHT as f32 * TILE_SIZE / 2.0;

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let texture = tile_map[y][x].clone();
            commands.spawn((
                Tile,
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

    // Pour éviter les chevauchements entre ressources, on stocke les positions déjà placées.
    // Chaque ressource est un carré de 10 tuiles de côté (MINERAL_SIZE/TILE_SIZE).
    // Deux ressources ne doivent pas être placées si leur différence en x ET en y est inférieure à 10.
    let mut placed_positions: Vec<(usize, usize)> = Vec::new();

    // Placer 150 minerais (le taux a été divisé par 2)
    let mut placed_minerals = 0;
    while placed_minerals < 150 {
        // Génération dans une plage qui garantit que le sprite (de 10 tuiles de côté) reste dans la carte.
        let candidate_x = rng.gen_range(RESOURCE_HALF_TILES..(WIDTH - RESOURCE_HALF_TILES));
        let candidate_y = rng.gen_range(RESOURCE_HALF_TILES..(HEIGHT - RESOURCE_HALF_TILES));

        // Vérification que le sprite ne chevauche pas un obstacle
        let mut overlaps_obstacle = false;
        for j in (candidate_y - RESOURCE_HALF_TILES)..(candidate_y + RESOURCE_HALF_TILES) {
            for i in (candidate_x - RESOURCE_HALF_TILES)..(candidate_x + RESOURCE_HALF_TILES) {
                if tile_map[j][i] == obstacle_handle {
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

        // Vérification qu'il n'y a pas déjà une ressource à proximité.
        // On considère qu'il y a chevauchement si la différence en x ET en y est inférieure à 10 tuiles.
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

        // Tout est OK, on peut placer le minerai
        commands.spawn((
            ResourcePoint,
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

    // Placer 50 énergies (taux divisé par 2)
    let mut placed_energy = 0;
    while placed_energy < 50 {
        let candidate_x = rng.gen_range(RESOURCE_HALF_TILES..(WIDTH - RESOURCE_HALF_TILES));
        let candidate_y = rng.gen_range(RESOURCE_HALF_TILES..(HEIGHT - RESOURCE_HALF_TILES));

        // Vérification que le sprite ne chevauche pas un obstacle
        let mut overlaps_obstacle = false;
        for j in (candidate_y - RESOURCE_HALF_TILES)..(candidate_y + RESOURCE_HALF_TILES) {
            for i in (candidate_x - RESOURCE_HALF_TILES)..(candidate_x + RESOURCE_HALF_TILES) {
                if tile_map[j][i] == obstacle_handle {
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

        // Vérification qu'il n'y a pas déjà une ressource à proximité.
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

        // Placement de l'énergie
        commands.spawn((
            ResourcePoint,
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
}
