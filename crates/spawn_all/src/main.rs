use bevy::prelude::*;

mod cam;
use cam::*;

pub mod agent;
pub use agent::*;

pub mod util;
pub use util::*;

use rand::prelude::*;

pub const LEVEL_WIDTH: f32 = 10000.0;
pub const LEVEL_HEIGHT: f32 = 20000.0;

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
        .add_plugin(CamPlugin)
        .insert_resource(Game::new())
        .add_startup_system(setup)
        // .add_system(move_character)
        .run();
}

fn setup(mut commands: Commands, mut game: ResMut<Game>, time: Res<Time>) {
    // commands
    //     .spawn_bundle(OrthographicCameraBundle::new_2d())
    //     .insert(Cam::default());

    commands
        .spawn_bundle(OrthographicCameraBundle {
            transform: Transform::from_translation(Vec3::new(LEVEL_WIDTH / 2.0, 0.0, 10.0)),

            orthographic_projection: OrthographicProjection {
                scale: 1.0,
                far: 100000.0,
                near: -100000.0,
                ..Default::default()
            },
            ..OrthographicCameraBundle::new_2d()
        })
        .insert(Cam::default());

    let world_size = Vec2::new(LEVEL_WIDTH, LEVEL_HEIGHT);
    let main_creature_size = Vec2::new(10.0, 10.);
    let floor_size = Vec2::new(LEVEL_WIDTH, 2000.);

    // ocean
    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(0.25, 0.25, 0.75),
            custom_size: Some(world_size),
            ..Default::default()
        },
        transform: Transform::from_translation(Vec3::new(
            LEVEL_WIDTH / 2.0,
            world_size.y / 2.0 - 1000.0,
            0.0,
        )),
        ..Default::default()
    });

    // floor
    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(0.35, 0.25, 0.25),
            custom_size: Some(floor_size),
            ..Default::default()
        },
        transform: Transform::from_translation(Vec3::new(LEVEL_WIDTH / 2.0, -1010.0, 0.01)),
        ..Default::default()
    });

    // main character
    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(0.75, 0.25, 0.55),
            custom_size: Some(main_creature_size),

            ..Default::default()
        },
        transform: Transform::from_translation(Vec3::new(LEVEL_WIDTH / 2.0, 0.0, 0.1)),
        ..Default::default()
    });

    // spawn all agents
    let mut rng = rand::thread_rng();
    for (id, agent) in game.agents.iter() {
        let creature_size = Vec2::splat(agent.mass * 500.0);
        let creature_pos = agent.position * 10000.0;
        println!("creature pos: {:?}", creature_pos);

        commands
            .spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(rng.gen::<f32>(), rng.gen::<f32>(), rng.gen::<f32>()),
                    custom_size: Some(creature_size),

                    ..Default::default()
                },
                transform: Transform::from_translation(creature_pos.extend(0.09)),
                ..Default::default()
            })
            .insert(AgentId { kdtree_hash: *id });
    }
}
