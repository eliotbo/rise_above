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

pub const POS_MULT: f32 = 10000.0;

pub const BOOST_TIMER: f32 = 0.3;

pub const MAIN_CHARA_Z: f32 = 0.1;
pub const TOTAL_BOOST_TIME: f32 = 0.3;

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
        .add_system(main_character_inputs)
        .add_system(agent_movement)
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
    let main_creature_size = Vec2::new(10.0, 5.);
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
    let mut random_agent = Agent::gen_random(&GameStage::Bottom, 1); // the 1 is for the main character's id
    random_agent.position = Vec2::new(LEVEL_WIDTH / 2.0, 20.0);

    let mut transform =
        Transform::from_translation(Vec3::new(LEVEL_WIDTH / 2.0, 0.0, MAIN_CHARA_Z));

    transform.rotation = Quat::from_rotation_z(random_agent.look_at_angle);
    game.agents.insert(1, random_agent);

    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.75, 0.25, 0.55),
                custom_size: Some(main_creature_size),

                ..Default::default()
            },
            transform,
            ..Default::default()
        })
        .insert(MainCharacter { id: 1 });

    // spawn all agents
    let mut rng = rand::thread_rng();
    for (id, agent) in game.agents.iter() {
        let creature_size = Vec2::splat(agent.mass * 500.0);
        let creature_pos = agent.position;
        // println!("creature pos: {:?}", creature_pos);

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

#[derive(Component)]
pub struct MainCharacter {
    id: u32,
}

pub fn main_character_inputs(
    mut game: ResMut<Game>,
    // mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    mouse_click: Res<Input<MouseButton>>,
    mut query: Query<(&mut Transform, &MainCharacter)>,

    time: Res<Time>,
) {
    // let main_character = game.agents.get(&game.main_character_id).unwrap();
    // let agent = game.
    let (mut transform, main_char) = query.single_mut();
    let mut agent = game.agents.get_mut(&main_char.id).unwrap();

    if keyboard_input.pressed(KeyCode::S) {
        agent.acc = Acceleration::Backward;
    } else if keyboard_input.pressed(KeyCode::W) {
        agent.acc = Acceleration::Forward;
    } else {
        agent.acc = Acceleration::None;
    }

    if keyboard_input.pressed(KeyCode::A) && !keyboard_input.pressed(KeyCode::D) {
        agent.turning = Turning::Left;
    } else if keyboard_input.pressed(KeyCode::D) && !keyboard_input.pressed(KeyCode::A) {
        agent.turning = Turning::Right;
    } else {
        agent.turning = Turning::None;
    }

    if (mouse_click.just_pressed(MouseButton::Left) || keyboard_input.just_pressed(KeyCode::Space))
        && time.seconds_since_startup() as f32 - agent.boost_time > BOOST_TIMER
        && agent.energy_shots > 1.0
    {
        agent.boost = true;
        agent.boost_time = time.seconds_since_startup() as f32;
        agent.energy_shots -= 1.0;
    }
}

pub fn agent_movement(
    mut game: ResMut<Game>,
    time: Res<Time>,
    mut query: Query<(&mut Transform, &MainCharacter), Without<Cam>>,
    mut cam_query: Query<&mut Transform, With<Cam>>,
) {
    let mut rng = rand::thread_rng();
    // for (id, agent) in game.agents.iter_mut() {
    for (mut transform, main_char) in &mut query.iter_mut() {
        let mut agent = game.agents.get_mut(&main_char.id).unwrap();

        agent.compute_self_velocity();
        let velocity = agent.velocity;
        let mut new_velocity = velocity;

        let mut verlet_velocity = agent.position - agent.last_position;

        let mut new_position = agent.position;

        let mut acc = Vec2::ZERO;
        let mut turn_angle = 0.0;

        let forward = agent.compute_look_at_dir();
        let (left, right) = agent.compute_left_and_right_dir();

        let friction = 0.1;
        let timestep = 1. / 60.;

        // apply acceleration
        match agent.acc {
            Acceleration::Forward => {
                // new_velocity += agent_acc * agent.mass * time.delta_seconds();
                acc = 1.0 * forward;
            }
            Acceleration::Backward => {
                // new_velocity -= agent_acc * agent.mass * time.delta_seconds();
                acc = -1.0 * forward;
            }
            Acceleration::None => {}
        }
        // println!("scc : {:?}", acc);

        // apply turning
        let speed_dependence = 0.03;
        match agent.turning {
            Turning::Left => {
                // new_velocity += agent.turning_acceleration * agent.mass * time.delta_seconds();
                // acc += 1.0 * left * agent.velocity.length();
                // acc += 1.0 * left;
                turn_angle = 0.01 + agent.speed * speed_dependence;
            }
            Turning::Right => {
                // new_velocity -= agent.turning_acceleration * agent.mass * time.delta_seconds();
                // acc += 1.0 * right;
                // turn_angle = -0.01 - agent.speed / agent.mass * speed_dependence;
                turn_angle = -0.01 - agent.speed * speed_dependence;
            }
            Turning::None => {}
        }

        if agent.boost {
            let boost_value = boost_impulse(time.seconds_since_startup() as f32 - agent.boost_time);
            acc = acc * (1.0 + boost_value * 3.0);
        }

        // // apply velocity
        // let delta_pos = acc * agent.mass * timestep * timestep * 1000000.0;
        let delta_pos = acc * timestep * timestep * 10000.0;
        new_position += delta_pos; // time.delta_seconds().powf(2.0);

        // // // apply position
        // agent.velocity = new_position - agent.position;

        agent.last_position = agent.position;
        agent.position = new_position;
        agent.look_at_angle = (agent.look_at_angle + turn_angle) % (2.0 * std::f32::consts::PI);

        agent.speed = verlet_velocity.length();
        println!("speed : {:?}", agent.speed);
        // println!("agent.position : {:?}", agent.velocity);

        transform.translation = agent.position.extend(MAIN_CHARA_Z);
        // transform.rotate(Quat::from_rotation_z(turn_angle));
        transform.rotation = Quat::from_rotation_z(agent.look_at_angle);

        let mut cam_transform = cam_query.single_mut();
        cam_transform.translation.x = agent.position.x;
        cam_transform.translation.y = agent.position.y;
    }
}

pub fn boost_impulse(time: f32) -> f32 {
    let rise_time = 0.7 * TOTAL_BOOST_TIME;
    let fall_time = 0.3 * TOTAL_BOOST_TIME;
    if time < rise_time {
        return time / rise_time;
    } else if time < TOTAL_BOOST_TIME {
        return 1.0 - (time - rise_time) / fall_time;
    }
    return 0.0;
}
