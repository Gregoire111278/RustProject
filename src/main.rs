use bevy::prelude::*;
use noise::{NoiseFn, Perlin, Seedable};
use rand::Rng;

const TILE_SIZE: f32 = 4.0;
const WIDTH: usize = 600;
const HEIGHT: usize = 600;
const NEIGHBOR_RADIUS: usize = 1;  

#[derive(Component)]
struct Tile;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 1000.0),
        ..Default::default()
    });

    let seed = 42;
    let perlin = Perlin::new().set_seed(seed);
    let mut rng = rand::thread_rng();
    let offset_x = WIDTH as f32 * TILE_SIZE / 2.0;
    let offset_y = HEIGHT as f32 * TILE_SIZE / 2.0;

    let resource_clustering_factor = 0.4;

    let mut tile_map: Vec<Vec<Color>> = vec![vec![Color::WHITE; WIDTH]; HEIGHT];

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let value = perlin.get([x as f64 / 10.0, y as f64 / 10.0]);
            let color = if value > 0.60 {
                Color::rgb(1.0, 0.0, 0.0) 
            } else if value > 0.40 {
                Color::rgb(0.0, 1.0, 0.0) 
            } else if value > 0.20 {
                Color::rgb(1.0, 1.0, 0.0) 
            } else {
                Color::rgb(1.0, 1.0, 1.0) 
            };

            tile_map[y][x] = color;
        }
    }

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let mut resource_prob = 1.0; 
            let current_color = tile_map[y][x];

            for dy in -(NEIGHBOR_RADIUS as isize)..=(NEIGHBOR_RADIUS as isize) {
                for dx in -(NEIGHBOR_RADIUS as isize)..=(NEIGHBOR_RADIUS as isize) {
                    if dy == 0 && dx == 0 {
                        continue; 
                    }

                    let nx = (x as isize + dx).max(0).min((WIDTH - 1) as isize) as usize;
                    let ny = (y as isize + dy).max(0).min((HEIGHT - 1) as isize) as usize;

                    let neighbor_color = tile_map[ny][nx];
                    if neighbor_color == current_color {
                        resource_prob *= 1.2; 
                    }
                }
            }

            let mut rng = rand::thread_rng();
            if rng.gen::<f32>() > resource_prob {
                let value = perlin.get([x as f64 / 10.0, y as f64 / 10.0]);
                tile_map[y][x] = if value > 0.60 {
                    Color::rgb(1.0, 0.0, 0.0) 
                } else if value > 0.40 {
                    Color::rgb(0.0, 1.0, 0.0) 
                } else if value > 0.20 {
                    Color::rgb(1.0, 1.0, 0.0) 
                } else {
                    Color::rgb(1.0, 1.0, 1.0) 
                };
            }
        }
    }


    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let color = tile_map[y][x];

            commands.spawn((
                Tile,
                SpriteBundle {
                    sprite: Sprite {
                        color,
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
