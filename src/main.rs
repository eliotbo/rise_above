use bevy::prelude::*;

mod ai;
use ai::*;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Rise Above".to_string(),
            width: 900.,
            height: 900.,
            vsync: true,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(move_character)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(0.25, 0.25, 0.75),
            custom_size: Some(Vec2::new(50.0, 100.0)),
            ..Default::default()
        },
        ..Default::default()
    });
}

struct Character {
    position: Vec2,
    velocity: Vec2,
    acceleration: Vec2,
    mass: f32,
    energy: f32,
}

pub fn move_character(keyboard: Res<Input<KeyCode>>) {
    let pressed_w = keyboard.just_pressed(KeyCode::W);
    let pressed_s = keyboard.just_pressed(KeyCode::S);
    let pressed_a = keyboard.just_pressed(KeyCode::A);
    let pressed_d = keyboard.just_pressed(KeyCode::D);

    let pressed_space = keyboard.just_pressed(KeyCode::Space);

    match (pressed_w, pressed_s, pressed_a, pressed_d) {
        (true, false, false, false) => {
            // Move forward
        }
        (false, true, false, false) => {
            // Move backward
        }
        (false, false, true, false) => {
            // Move left
        }
        (false, false, false, true) => {
            // Move right
        }
        (true, true, false, false) => {
            // Move forward and left
        }
        (false, true, true, false) => {
            // Move forward and right
        }
        (true, false, false, true) => {
            // Move backward and left
        }
        (false, false, true, true) => {
            // Move backward and right
        }
        (false, false, false, false) => {
            // Stop moving
        }
        _ => {}
    }

    // let mut total_force = Vec2::new(0.0, 0.0);
}
