use bevy::prelude::*;
use noise::{NoiseFn, Perlin, Seedable};
use rand::Rng;

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

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let value = perlin.get([x as f64 / 10.0, y as f64 / 10.0]);
            let is_path = rng.gen_bool(0.90); // Augmentation des chemins neutres (75%)

            let color = if is_path {
                Color::rgb(0.9, 0.8, 0.6) // Chemin neutre (couleur sable)
            } else if value > 0.60 {
                Color::rgb(1.0, 0.0, 0.0) // Obstacle (rouge)
            } else if value > 0.40 {
                Color::rgb(0.0, 1.0, 0.0) // Minerais (vert)
            } else if value > 0.20 {
                Color::rgb(1.0, 1.0, 0.0) // Énergie (jaune)
            } else {
                Color::rgb(1.0, 1.0, 1.0) // Lieu d'intérêt scientifique (blanc)
            };

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
