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

// for debug_quad in query_debug.iter() {
//     commands.entity(debug_quad).despawn();
// }

// commands
//     .spawn_bundle(SpriteBundle {
//         sprite: Sprite {
//             color: Color::rgb(rng.gen::<f32>(), rng.gen::<f32>(), rng.gen::<f32>()),
//             custom_size: Some(Vec2::splat(10.0)),

//             ..Default::default()
//         },
//         transform: Transform::from_translation(
//             (agent.position + velocity_dir * 10.0).extend(1.09),
//         ),
//         ..Default::default()
//     })
//     .insert(DebugQuad);

pub struct MovementParams {
    pub friction: f32,
    pub throttle: f32,
    pub turning_speed_dependence: f32,
    pub backwards_mult: f32,
    pub boost_mult: f32,
    pub rest_turn_speed: f32,
    pub max_turn_speed: f32,
}

impl Default for MovementParams {
    fn default() -> Self {
        Self {
            friction: 0.2,
            turning_speed_dependence: 0.03,
            backwards_mult: 0.3,
            boost_mult: 3.0,
            rest_turn_speed: 0.02,
            max_turn_speed: 0.05,
            throttle: 500.0,
        }
    }
}

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
        .insert_resource(MovementParams::default())
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
    random_agent.last_position = random_agent.position;

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

pub fn main_character_inputs(
    mut game: ResMut<Game>,
    // mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    mouse_click: Res<Input<MouseButton>>,
    mut query: Query<&MainCharacter>,

    time: Res<Time>,
) {
    let main_char = query.single_mut();
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

    if (mouse_click.just_pressed(MouseButton::Left) || keyboard_input.pressed(KeyCode::Space))
        && time.seconds_since_startup() as f32 - agent.boost_time > BOOST_TIMER
        && agent.energy_shots > 1.0
    {
        agent.boost = true;
        agent.boost_time = time.seconds_since_startup() as f32;
        agent.energy_shots -= 1.0;
    }
}

#[derive(Component)]
pub struct DebugQuad;

pub fn agent_movement(
    // mut commands: Commands,
    // mut query_debug: Query<Entity, With<DebugQuad>>,
    mut game: ResMut<Game>,
    time: Res<Time>,
    mut query: Query<(&mut Transform, &MainCharacter), Without<Cam>>,

    mut cam_query: Query<&mut Transform, With<Cam>>,
    move_params: Res<MovementParams>,
) {
    // let mut rng = rand::thread_rng();
    // for (id, agent) in game.agents.iter_mut() {
    for (mut transform, main_char) in &mut query.iter_mut() {
        let mut agent = game.agents.get_mut(&main_char.id).unwrap();

        agent.compute_self_velocity();

        let timestep = time.delta_seconds() as f32;

        // let friction = 0.05;
        // let turning_speed_dependence = 0.03;
        // let backwards_mult = 0.3;
        // let boost_mult = 3.0;
        // let rest_turn_speed = 0.02;
        // let max_turn_speed = 0.05;
        // let throttle = 10000.0;

        let friction = move_params.friction;
        let turning_speed_dependence = move_params.turning_speed_dependence;
        let backwards_mult = move_params.backwards_mult;
        let boost_mult = move_params.boost_mult;
        let rest_turn_speed = move_params.rest_turn_speed;
        let max_turn_speed = move_params.max_turn_speed;
        let throttle = move_params.throttle;

        let verlet_velocity = agent.position - agent.last_position;
        agent.speed = verlet_velocity.length();

        let velocity_dir = agent.forward_dir();

        let mut new_position = agent.position;

        let mut acc = Vec2::ZERO;
        let mut turn_angle = 0.0;

        let forward = agent.compute_look_at_dir();

        match agent.acc {
            Acceleration::Forward => {
                acc = forward;
            }
            Acceleration::Backward => {
                acc = -forward * backwards_mult;
            }
            Acceleration::None => {}
        }

        let mut boost_value = 0.0;
        if agent.boost {
            boost_value = boost_impulse(time.seconds_since_startup() as f32 - agent.boost_time);
            acc = acc * (1.0 + boost_value * boost_mult);
        }

        let (left, right) = agent.compute_left_and_right_dir();

        // apply turning

        match agent.turning {
            Turning::Left => {
                let speed_turn = agent.speed * turning_speed_dependence * (1.0 - boost_value);
                turn_angle =
                    rest_turn_speed + speed_turn.clamp(0.0, max_turn_speed - rest_turn_speed);
            }
            Turning::Right => {
                let speed_turn = agent.speed * turning_speed_dependence * (1.0 - boost_value);
                turn_angle =
                    -rest_turn_speed - speed_turn.clamp(0.0, max_turn_speed - rest_turn_speed);
            }
            Turning::None => {}
        }

        let friction_force = -friction * verlet_velocity.length().powf(2.0) * velocity_dir;

        acc += friction_force;

        new_position += verlet_velocity + acc * timestep * timestep * throttle;

        agent.speed = (new_position - agent.position).length();

        agent.last_position = agent.position;

        agent.position = new_position;

        agent.look_at_angle = (agent.look_at_angle + turn_angle) % (2.0 * std::f32::consts::PI);

        transform.translation = agent.position.extend(MAIN_CHARA_Z);

        transform.rotation = Quat::from_rotation_z(agent.look_at_angle);

        // TODO: smooth out the camera
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
