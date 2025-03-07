use bevy::prelude::*;
use noise::{NoiseFn, Perlin, Seedable};
use rand::Rng;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    let seed = 42;
    let perlin = Perlin::new().set_seed(seed);
    let width = 30;
    let height = 15;
    let mut rng = rand::thread_rng();

    for y in 0..height {
        for x in 0..width {
            let value = perlin.get([x as f64 / 10.0, y as f64 / 10.0]);

            let is_path = rng.gen_bool(0.5); 
            let color = if is_path {
                Color::rgb(0.5, 0.5, 0.5) 
            } else if value > 0.8 {
                Color::rgb(1.0, 0.0, 0.0)
            } else if value > 0.6 {
                Color::rgb(0.0, 1.0, 0.0) 
            } else if value > 0.4 {
                Color::rgb(1.0, 1.0, 0.0) 
            } else {
                Color::rgb(1.0, 1.0, 1.0) 
            };

            commands.spawn(SpriteBundle {
                sprite: Sprite {
                    color,
                    custom_size: Some(Vec2::new(32.0, 32.0)),
                    ..Default::default()
                },
                transform: Transform::from_xyz(x as f32 * 32.0, y as f32 * 32.0, 0.0),
                ..Default::default()
            });
        }
    }
}
