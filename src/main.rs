use bevy::prelude::*;
use noise::{NoiseFn, Perlin, Seedable};
use rand::Rng;

const TILE_SIZE: f32 = 4.0;
const WIDTH: usize = 370;
const HEIGHT: usize = 190;
const MINERAL_SIZE: f32 = TILE_SIZE * 10.0;
const RESOURCE_HALF_TILES: usize = (MINERAL_SIZE / TILE_SIZE / 2.0) as usize;

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

    let seed = 42;
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
                let value = perlin.get([x as f64 / 16.0, y as f64 / 16.0]);
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

    let mut placed_positions: Vec<(usize, usize)> = Vec::new();

    let mut placed_minerals = 0;
    while placed_minerals < 150 {
        let candidate_x = rng.gen_range(RESOURCE_HALF_TILES..(WIDTH - RESOURCE_HALF_TILES));
        let candidate_y = rng.gen_range(RESOURCE_HALF_TILES..(HEIGHT - RESOURCE_HALF_TILES));

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

    let mut placed_energy = 0;
    while placed_energy < 50 {
        let candidate_x = rng.gen_range(RESOURCE_HALF_TILES..(WIDTH - RESOURCE_HALF_TILES));
        let candidate_y = rng.gen_range(RESOURCE_HALF_TILES..(HEIGHT - RESOURCE_HALF_TILES));

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
