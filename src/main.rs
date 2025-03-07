use bevy::prelude::*;
use noise::{NoiseFn, Perlin, Seedable};

const TILE_SIZE: f32 = 4.0;
const WIDTH: usize = 600;
const HEIGHT: usize = 600;

#[derive(Component)]
struct Tile;

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

    let desert_handle = asset_server.load("desert.jpg");
    let obstacle_handle = asset_server.load("obstacle.jpg");
    let grass_handle = asset_server.load("desert.jpg");
    let forest_handle = asset_server.load("desert.jpg");
    let snow_handle = asset_server.load("desert.jpg");

    let mut tile_map: Vec<Vec<Handle<Image>>> = vec![vec![desert_handle.clone(); WIDTH]; HEIGHT];

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
                let value = perlin.get([x as f64 / 10.0, y as f64 / 10.0]);
                if value > 0.60 {
                    tile_map[y][x] = grass_handle.clone();
                }
            }
        }
    }

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            if tile_map[y][x] == desert_handle {
                let value = perlin.get([x as f64 / 10.0, y as f64 / 10.0]);
                if value > 0.40 {
                    tile_map[y][x] = forest_handle.clone();
                }
            }
        }
    }

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            if tile_map[y][x] == desert_handle {
                let value = perlin.get([x as f64 / 8.0, y as f64 / 8.0]);
                if value > 0.60 {
                    tile_map[y][x] = snow_handle.clone();
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
}
