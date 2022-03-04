// mod cam;

// pub mod agent;
// pub mod encoding;
// pub mod inputs;
// mod libaaa;

// mod lib;

pub use agent::*;
use cam::*;
pub use encoding::*;
pub use rise_above::*;

// pub mod util;
pub use util::*;

pub use inputs::*;

// pub use libaaa::*;

use bevy::prelude::*;
use rand::prelude::*;

// use std::fs::File;
// use std::io::Read;
// use std::io::Write;
// use std::path::PathBuf;

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

// use bevy_inspector_egui::{Inspectable, InspectorPlugin};

// #[derive(Inspectable)]
pub struct MovementParams {
    // #[inspectable(min = 0.00, max = 1.0, speed = 0.001)]
    pub friction1: f32,

    // #[inspectable(min = 0.00, max = 1.0, speed = 0.001)]
    pub friction2: f32,

    // #[inspectable(min = 10.0, max = 2000.0, speed = 1.0)]
    pub throttle: f32,

    // #[inspectable(min = 0.0001, max = 0.1, speed = 0.0001)]
    pub turning_speed_dependence: f32,

    // #[inspectable(min = 0.005, max = 1.0, speed = 0.0001)]
    pub backwards_mult: f32,

    // #[inspectable(min = 1.4, max = 50.0, speed = 0.2)]
    pub boost_mult: f32,

    // #[inspectable(min = 0.005, max = 0.5, speed = 0.0005)]
    pub rest_turn_speed: f32,

    // #[inspectable(min = 0.005, max = 0.5, speed = 0.0001)]
    pub max_turn_speed: f32,

    // #[inspectable(min = 0.02, max = 3.0, speed = 0.001)]
    pub time_between_boosts: f32,

    // #[inspectable(min = 0.02, max = 3.0, speed = 0.001)]
    // pub downcurrent: f32,
    // #[inspectable(min = 0.1, max = 100.0, speed = 0.01)]
    pub bottom_bounce: f32,
}

impl Default for MovementParams {
    fn default() -> Self {
        Self {
            friction1: 0.9,
            friction2: 0.9,
            turning_speed_dependence: 0.03,
            backwards_mult: 0.3,
            boost_mult: 5.0,
            rest_turn_speed: 0.02,
            max_turn_speed: 0.05,
            throttle: 50.0,
            time_between_boosts: 2.0,
            // downcurrent: 0.1,
            bottom_bounce: 2.0,
        }
    }
}

impl MovementParams {
    pub fn stage1() -> Self {
        Self {
            friction1: 0.2,
            friction2: 0.2,
            turning_speed_dependence: 0.03,
            backwards_mult: 0.3,
            boost_mult: 5.0,
            rest_turn_speed: 0.02,
            max_turn_speed: 0.05,
            throttle: 150.0,
            time_between_boosts: 1.0,
            // downcurrent: 0.5,
            bottom_bounce: 10.0,
        }
    }

    fn stage2() -> Self {
        Self {
            friction1: 0.1,
            friction2: 0.13,
            turning_speed_dependence: 0.03,
            backwards_mult: 0.3,
            boost_mult: 13.0,
            rest_turn_speed: 0.013,
            max_turn_speed: 0.05,
            throttle: 200.0,
            time_between_boosts: 0.5,
            // downcurrent: 2.0,
            bottom_bounce: 10.0,
        }
    }

    fn stage3() -> Self {
        Self {
            friction1: 0.09,
            friction2: 0.013,
            turning_speed_dependence: 0.05,
            backwards_mult: 0.3,
            boost_mult: 5.0,
            rest_turn_speed: 0.02,
            max_turn_speed: 0.1,
            throttle: 1500.0,
            time_between_boosts: 0.25,
            // downcurrent: 1.0,
            bottom_bounce: 50.0,
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
        // .add_plugin(InspectorPlugin::<MovementParams>::new())
        .add_event::<CollisionEvent>()
        .insert_resource(Cursor::default())
        .insert_resource(MovementParams::stage1())
        .insert_resource(Game::new())
        .insert_resource(KdTrees::new())
        .add_startup_system(setup)
        .add_system(main_character_inputs)
        .add_system(main_char_movement)
        .add_system(agents_movement)
        .add_system(record_mouse_events_system)
        .add_system(collisions)
        .add_system(see)
        .add_system(update_agent_kdtree)
        .add_system(forget)
        .add_system(agent_decisions)
        .add_system(agent_action)
        .add_system(update_agent_properties.exclusive_system().at_end())
        .add_system(energy_ground_state)
        .add_system(winning_condition)
        .add_system(send_guardians)
        //
        // .add_system(agent_movement_debug)
        // .add_system(load_character)
        // .add_system(load_character_auto)
        .run();
}

pub fn send_guardians(mut game: ResMut<Game>, time: Res<Time>) {
    if time.seconds_since_startup() > 0.0 {
        let main_char = game.agents.get(&1).unwrap();
        let main_char_height = main_char.position.y;
        if main_char_height > LEVEL_HEIGHT / 2.0 {
            for (id, agent) in game.agents.iter_mut() {
                //
                // guardians
                if *id >= 20 && *id < 40 {
                    agent.goal = Goal::Bully(1); // main_char.id;
                    agent.goal_time = time.seconds_since_startup() as f32;
                }
            }
        } else {
            for (id, agent) in game.agents.iter_mut() {
                //
                // guardians
                if *id >= 20 && *id < 35 {
                    agent.goal = Goal::GoTo(agent.guardian_pos); // main_char.id;
                    agent.goal_time = time.seconds_since_startup() as f32;
                } else if *id < 40 {
                    agent.goal = Goal::Bully(1);
                    agent.goal_time = time.seconds_since_startup() as f32;
                }
            }
        }
    }
}

pub fn update_agent_kdtree(mut kdtrees: ResMut<KdTrees>, mut game: ResMut<Game>) {
    let mut rng = rand::thread_rng();
    for (_id, mut agent) in game.agents.iter_mut() {
        if !agent.position.y.is_finite() {
            agent.position = Vec2::new(
                rng.gen::<f32>() * LEVEL_WIDTH,
                rng.gen::<f32>() * LEVEL_HEIGHT,
            );
        }
    }

    kdtrees.gen_agent_kdtree(&game.agents);
}

pub fn update_movement_params(mut movement_params: ResMut<MovementParams>, game: Res<Game>) {
    let main_character = game.agents.get(&1).unwrap();
    if main_character.energy > 0.2 {
        *movement_params = MovementParams::stage3();
    } else if main_character.energy > 0.12 {
        *movement_params = MovementParams::stage2();
    }
}

fn setup(
    mut commands: Commands,
    mut game: ResMut<Game>,
    mut kdtrees: ResMut<KdTrees>,
    asset_server: Res<AssetServer>,
    // time: Res<Time>,
) {
    // commands
    //     .spawn_bundle(OrthographicCameraBundle::new_2d())
    //     .insert(Cam::default());
    kdtrees.as_mut().populate(game.as_ref());

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
    let floor_size = Vec2::new(LEVEL_WIDTH + 4000.0, 2000.);
    let wall_size = Vec2::new(2000.0, LEVEL_HEIGHT);

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

    // Walls
    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(0.35, 0.25, 0.25),
            custom_size: Some(wall_size),
            ..Default::default()
        },
        transform: Transform::from_translation(Vec3::new(
            LEVEL_WIDTH + 1000.0,
            world_size.y / 2.0 - 1000.0,
            0.01,
        )),
        ..Default::default()
    });

    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(0.35, 0.25, 0.25),
            custom_size: Some(wall_size),
            ..Default::default()
        },
        transform: Transform::from_translation(Vec3::new(
            -1000.0,
            world_size.y / 2.0 - 1000.0,
            0.01,
        )),
        ..Default::default()
    });

    let contents = include_str!("notme.cha");

    let loaded_character: CharacterSaveFormat = serde_json::from_str(&contents).unwrap();

    let nodes = loaded_character
        .data
        .iter()
        .map(|node| node.pos * MASS_MULT)
        .collect::<Vec<_>>();

    // let character: MarkerInstanceMatData = loaded_character.clone().into();

    //////////////////// main character////////////////////////////////////////////////////////////////////////
    // the 1 is for the main character's id
    let mut main_agent = Agent::gen_random(&GameStage::Bottom, 1);
    // main_agent.position = Vec2::new(LEVEL_WIDTH / 2.0, 20.0);
    // main_agent.position = Vec2::new(LEVEL_WIDTH / 2.0, 20.0);
    main_agent.position = Vec2::new(LEVEL_WIDTH / 2.0, LEVEL_HEIGHT - 1000.0 - 20.0);
    main_agent.last_position = main_agent.position;
    // main_agent.mass = 0.1;
    // main_agent.update_mass_properties();
    // main_agent.radius = main_agent.mass * MASS_MULT * 0.5;

    let atom_size = Vec2::splat(ATOM_MULT * main_agent.mass * MASS_MULT);
    main_agent.body = nodes
        .iter()
        .enumerate()
        .map(|(k, node)| Body {
            atom_pos: *node * main_agent.mass,
            rotation: Quat::from_rotation_z(main_agent.look_at_angle),
            atom_size: atom_size.length(),
            acceleration: Vec2::new(0.0, 0.0),
            entity: None,
            is_used: false,
        })
        .collect::<Vec<_>>();
    // main_agent.mass = 0.05;

    let mut transform = Transform::from_translation(Vec3::new(
        main_agent.position.x,
        main_agent.position.y,
        MAIN_CHARA_Z,
    ));

    transform.rotation =
        Quat::from_rotation_z(main_agent.look_at_angle + std::f32::consts::PI / 1.0);

    // let transform = Transform::from_translation(Vec3::new(LEVEL_WIDTH - 10.0, 0.0, MAIN_CHARA_Z));
    main_agent.position = transform.translation.truncate();
    main_agent.last_position = main_agent.position;

    let core_id = commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.75, 0.25, 0.55),
                custom_size: Some(main_creature_size * 0.3),

                ..Default::default()
            },
            transform,
            ..Default::default()
        })
        .insert(AgentId { kdtree_hash: 1 })
        .insert(MainCharacter { id: 1 })
        .id();
    main_agent.entity = Some(core_id);

    println!("core id: {}", nodes[0]);
    println!("core id: {}", 0.49 * MASS_MULT * main_agent.mass);

    nodes.iter().enumerate().for_each(|(k, pos)| {
        if pos.length() < 0.49 * MASS_MULT {
            let transform = Transform::from_translation(pos.extend(4.0) * main_agent.mass);

            let child_id = commands
                .spawn_bundle(SpriteBundle {
                    sprite: Sprite {
                        color: Color::rgb(0.75, 0.75, 0.55),
                        custom_size: Some(atom_size),

                        ..Default::default()
                    },
                    transform,
                    ..Default::default()
                })
                .insert(Atom)
                .id();

            main_agent.body[k].entity = Some(child_id);
            main_agent.body[k].is_used = true;

            commands.entity(core_id).push_children(&[child_id]);
        }
    });
    game.agents.insert(1, main_agent);

    //////////////////// main character ////////////////////////////////////////////////////////////////////////

    ////////////////////////////// spawn all npcs ////////////////////////////////////////////////
    let mut rng = rand::thread_rng();
    for (id, mut agent) in game.agents.iter_mut() {
        if id == &1 {
            continue;
        }
        let creature_pos = agent.position;
        // println!("creature pos: {:?}", creature_pos);

        let color = Color::rgb(rng.gen::<f32>(), rng.gen::<f32>(), rng.gen::<f32>());

        // TODO: remove, only useful for testing
        let creature_size = Vec2::splat(MASS_MULT * agent.mass * 0.001);

        // agent.radius = agent.mass * MASS_MULT * 0.5;

        let mut agent_trans = Transform::from_translation(creature_pos.extend(0.09));

        agent_trans.rotation = Quat::from_rotation_z(agent.look_at_angle);

        let npc_entity = commands
            .spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    color,
                    custom_size: Some(creature_size),

                    ..Default::default()
                },
                transform: agent_trans,
                ..Default::default()
            })
            .insert(NPC)
            .insert(AgentId { kdtree_hash: *id })
            .id();

        agent.entity = Some(npc_entity);

        let atom_size = Vec2::splat(ATOM_MULT * agent.mass * MASS_MULT);

        agent.body = nodes
            .iter()
            .enumerate()
            .map(|(k, node)| Body {
                atom_pos: *node * agent.mass,
                rotation: Quat::from_rotation_z(agent.look_at_angle),
                atom_size: atom_size.length(),
                acceleration: Vec2::new(0.0, 0.0),
                entity: None,
                is_used: false,
            })
            .collect::<Vec<_>>();
        // main_agent.mass = 0.05;

        nodes.iter().enumerate().for_each(|(k, pos)| {
            if pos.length() < 0.49 * MASS_MULT {
                let mut transform = Transform::from_translation(pos.extend(4.0) * agent.mass);

                let npc_child_id = commands
                    .spawn_bundle(SpriteBundle {
                        sprite: Sprite {
                            color,
                            custom_size: Some(atom_size),

                            ..Default::default()
                        },
                        transform,
                        ..Default::default()
                    })
                    // .insert(MainCharacter { id: 1 })
                    .insert(Atom)
                    .id();

                agent.body[k].entity = Some(npc_child_id);
                agent.body[k].is_used = true;

                commands.entity(npc_entity).push_children(&[npc_child_id]);
            }
        });
    }
    ////////////////////////////// spawn all npcs ////////////////////////////////////////////////

    ////////////////////////////// spawn food ////////////////////////////////////////////////
    let mut rng = rand::thread_rng();
    // println!("food len: {:?}, ", game.foods.len());
    for (id, mut food) in game.foods.iter_mut() {
        let food_pos = food.position;
        // println!("creature pos: {:?}", creature_pos);

        let color = Color::rgb(
            rng.gen::<f32>() * 0.1,
            rng.gen::<f32>() * 0.1,
            rng.gen::<f32>() * 0.6,
        );

        // TODO: remove, only useful for testing
        let food_size = Vec2::splat(MASS_MULT * food.mass.powf(0.5) * 0.1);
        // let food_size = Vec2::splat(5.0);

        // food.radius = food.mass * MASS_MULT * 0.5;

        let food_trans = Transform::from_translation(food_pos.extend(0.03));
        // println!("food: {:?}, ", food_trans.translation);

        let _food_entity = commands
            .spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    color,
                    custom_size: Some(food_size),

                    ..Default::default()
                },
                transform: food_trans,
                ..Default::default()
            })
            .insert(FoodComp)
            // .insert(foodId { kdtree_hash: *id })
            .id();
    }
    ////////////////////////////// spawn food ////////////////////////////////////////////////

    ////////////////////////////// text /////////////////////////////
    let text_style = TextStyle {
        font: asset_server.load("fonts/Roboto-Regular.ttf"),
        font_size: 44.0,
        color: Color::BLACK,
    };
    let text_alignment = TextAlignment {
        vertical: VerticalAlign::Bottom,
        horizontal: HorizontalAlign::Center,
    };

    commands
        .spawn_bundle(Text2dBundle {
            text: Text::with_section(
                "Focus on your breathing and rise above the surface.              The moshpit is essential.             Once a friend, twice a foe.",
                text_style.clone(),
                text_alignment,
            ),
            transform: Transform::from_translation(Vec3::new(
                LEVEL_WIDTH / 2.0 + 800.0,
                -100.0,
                10.0,
            )),
            ..Default::default()
        })
        .insert(StartText);

    let text_style = TextStyle {
        font: asset_server.load("fonts/Roboto-Regular.ttf"),
        font_size: 44.0,
        color: Color::BLACK,
    };
    let text_alignment = TextAlignment {
        vertical: VerticalAlign::Bottom,
        horizontal: HorizontalAlign::Center,
    };

    commands
        .spawn_bundle(Text2dBundle {
            text: Text::with_section(
                "Use Space or right click to boost. Use WASD or the left click to steer",
                text_style.clone(),
                text_alignment,
            ),
            transform: Transform::from_translation(Vec3::new(
                LEVEL_WIDTH / 2.0 - 800.0,
                -100.0,
                10.0,
            )),
            ..Default::default()
        })
        .insert(StartText);
}

pub fn energy_ground_state(mut game: ResMut<Game>, time: Res<Time>) {
    // let mut game = Game::new();
    for (id, mut agent) in game.agents.iter_mut() {
        if time.seconds_since_startup() as f32 - agent.last_collision_time > 1.5 {
            if agent.energy < 1.0 {
                agent.energy = agent.energy + ENERGY_REGAIN_RATE;
            } else {
                agent.energy =
                    agent.energy - ENERGY_DECAY_RATE * (agent.energy - ENERGY_GROUND_STATE);
            }
            if *id == 1 {
                // println!("energy: {:?}", agent.energy);
            }
        }
    }
}

pub fn winning_condition(
    game: ResMut<Game>,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    let agent = game.agents.get(&1).unwrap();

    if agent.position.y > LEVEL_HEIGHT - 1000.0 && !game.won {
        let text_style = TextStyle {
            font: asset_server.load("fonts/Roboto-Regular.ttf"),
            font_size: 44.0,
            color: Color::BLACK,
        };
        let text_alignment = TextAlignment {
            vertical: VerticalAlign::Bottom,
            horizontal: HorizontalAlign::Center,
        };

        println!("you won!");

        let mut text = Text::with_section("You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win! You win!", text_style.clone(), text_alignment);

        if time.seconds_since_startup() < 40.0 {
            text = Text::with_section(
                "You win! ... but you didn't experience the game, you might want to restart... or not. You did win... so...",
                text_style.clone(),
                text_alignment,
            );
        }

        commands
            .spawn_bundle(Text2dBundle {
                text,
                transform: Transform::from_translation(Vec3::new(
                    LEVEL_WIDTH / 2.0,
                    LEVEL_HEIGHT + 100.0 - 1000.0,
                    10.0,
                )),
                ..Default::default()
            })
            .insert(StartText);
    }
}

pub fn main_character_inputs(
    mut game: ResMut<Game>,
    // mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    mouse_click: Res<Input<MouseButton>>,
    mut query: Query<&MainCharacter>,

    time: Res<Time>,
    move_params: Res<MovementParams>,
    cursor: Res<Cursor>,
) {
    let main_char = query.single_mut();
    let mut agent = game.agents.get_mut(&main_char.id).unwrap();

    if keyboard_input.pressed(KeyCode::S) {
        agent.acc = Acceleration::Backward;
        agent.main_char_target_pos = None;
    } else if keyboard_input.pressed(KeyCode::W) {
        agent.acc = Acceleration::Forward;
        agent.main_char_target_pos = None;
    } else {
        agent.acc = Acceleration::None;
    }

    if keyboard_input.pressed(KeyCode::A) && !keyboard_input.pressed(KeyCode::D) {
        agent.turning = Turning::Left(1.0);
        agent.main_char_target_pos = None;
    } else if keyboard_input.pressed(KeyCode::D) && !keyboard_input.pressed(KeyCode::A) {
        agent.turning = Turning::Right(1.0);
        agent.main_char_target_pos = None;
    } else {
        agent.turning = Turning::None;
    }

    if (mouse_click.just_pressed(MouseButton::Right) || keyboard_input.pressed(KeyCode::Space))
        && time.seconds_since_startup() as f32 - agent.boost_time > move_params.time_between_boosts
    // && agent.energy > 1.0
    {
        agent.boost = true;
        agent.boost_time = time.seconds_since_startup() as f32;
        // agent.energy -= 1.0;
        agent.main_char_target_pos = None;
    }

    if mouse_click.pressed(MouseButton::Left) {
        agent.main_char_target_pos = Some(cursor.position);
    }

    if let Some(pos) = agent.main_char_target_pos {
        let target_dir = pos - agent.position;
        if target_dir != Vec2::ZERO {
            // let target_angle = target_dir.y.atan2(target_dir.x);
            let look_at_dir = agent.compute_look_at_dir();
            let look_at_90 = Vec2::new(-look_at_dir.y, look_at_dir.x);

            let dot_dirs = look_at_90.dot(target_dir.normalize());

            // println!("look_at_dir: {:?}", dot_dirs);

            // let delta_angle = target_angle - agent.look_at_angle - std::f32::consts::PI / 2.0;

            if dot_dirs < 0.0 {
                agent.turning = Turning::Right(dot_dirs.abs());
            } else {
                agent.turning = Turning::Left(dot_dirs.abs());
            }

            agent.acc = Acceleration::Forward;
            // println!("target angle: {:?}", dot_dirs);
        }
    }
}

#[derive(Component)]
pub struct DebugQuad;

pub fn agent_movement_debug(
    mut game: ResMut<Game>,
    mut query: Query<(&mut Transform, &MainCharacter), Without<Cam>>,

    mut cam_query: Query<&mut Transform, With<Cam>>,
    // move_params: Res<MovementParams>,
) {
    for (mut transform, main_char) in &mut query.iter_mut() {
        let mut agent = game.agents.get_mut(&main_char.id).unwrap();
        // let forward = Vec2::new(0.0, 1.0) * 0.5;
        let forward = agent.compute_look_at_dir();
        let left = Vec2::new(-1.0, 0.0) * 0.5;
        let mut acc = Vec2::ZERO;

        let mut turn_angle = 0.0;

        match agent.acc {
            Acceleration::Forward => {
                acc = forward;
            }
            Acceleration::Backward => {
                acc = -forward;
            }
            Acceleration::None => {}
        }

        match agent.turning {
            Turning::Left(mut delta_angle) => {
                // acc = acc + left;
                turn_angle = 0.03;
            }
            Turning::Right(mut delta_angle) => {
                // acc = acc - left;
                turn_angle = -0.03;
            }
            Turning::None => {}
        }

        agent.position += acc;
        transform.translation = agent.position.extend(2.2);

        agent.look_at_angle = (agent.look_at_angle + turn_angle) % (2.0 * std::f32::consts::PI);

        transform.translation = agent.position.extend(MAIN_CHARA_Z);

        transform.rotation = Quat::from_rotation_z(agent.look_at_angle);

        let mut cam_transform = cam_query.single_mut();
        cam_transform.translation.x = agent.position.x;
        cam_transform.translation.y = agent.position.y;
    }
}

pub fn main_char_movement(
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

        let friction1 = move_params.friction1;
        let friction2 = move_params.friction2;
        let turning_speed_dependence = move_params.turning_speed_dependence;
        let backwards_mult = move_params.backwards_mult;
        let boost_mult = move_params.boost_mult + (agent.mass / 0.05);
        let rest_turn_speed = move_params.rest_turn_speed;
        let max_turn_speed = move_params.max_turn_speed;
        let throttle = move_params.throttle;
        // let downcurrent = move_params.downcurrent ;

        let bottom_bounce = move_params.bottom_bounce;

        let downcurrent = 0.1 + agent.position.y / LEVEL_HEIGHT * 3.0;

        let verlet_velocity = agent.position - agent.last_position;
        agent.speed = verlet_velocity.length();

        let velocity_dir = agent.forward_dir();

        let mut new_position = agent.position;

        let mut acc = Vec2::ZERO;
        let mut turn_angle = 0.0;

        let forward = agent.compute_look_at_dir();

        match agent.acc {
            Acceleration::Forward => {
                acc = forward * agent.energy;
            }
            Acceleration::Backward => {
                acc = -forward * backwards_mult;
            }
            Acceleration::None => {}
        }

        let mut boost_value = 0.0;
        if agent.boost {
            boost_value = boost_impulse(time.seconds_since_startup() as f32 - agent.boost_time);
            acc = acc * (1.0 + boost_value * boost_mult * agent.energy);
        }

        // let (left, right) = agent.compute_left_and_right_dir();

        // apply turning

        let soft_angular = 0.5;

        match agent.turning {
            Turning::Left(mut delta_angle) => {
                if delta_angle < soft_angular {
                    delta_angle = delta_angle / soft_angular;
                } else {
                    delta_angle = 1.0;
                }
                let speed_turn = agent.speed * turning_speed_dependence * (1.0 - boost_value);
                turn_angle = delta_angle
                    * (rest_turn_speed + speed_turn.clamp(0.0, max_turn_speed - rest_turn_speed));
            }
            Turning::Right(mut delta_angle) => {
                if delta_angle < soft_angular {
                    delta_angle = delta_angle / soft_angular;
                } else {
                    delta_angle = 1.0;
                }
                let speed_turn = agent.speed * turning_speed_dependence * (1.0 - boost_value);
                turn_angle =
                    -rest_turn_speed - speed_turn.clamp(0.0, max_turn_speed - rest_turn_speed);
            }
            Turning::None => {}
        }

        let friction_force = -friction1 * verlet_velocity.length() * velocity_dir
            - friction2 * verlet_velocity.length().powf(2.0) * velocity_dir;

        let downcurrent_force = downcurrent * Vec2::new(0.0, -1.0);

        acc += friction_force + downcurrent_force;

        new_position += verlet_velocity + acc * timestep * timestep * throttle;

        // cannot fall below the ground
        let bottom_most_pos = agent.radius;
        if new_position.y < bottom_most_pos {
            new_position.y = bottom_most_pos + bottom_bounce;
        }

        // bounce off the walls
        // TODO: make wall_bounce part of the movement params
        let wall_bounce = 4.0;
        let left_most_pos = agent.radius;
        if new_position.x < left_most_pos {
            new_position.x = left_most_pos + wall_bounce;
        }

        let right_most_pos = LEVEL_WIDTH - agent.radius;
        if new_position.x > right_most_pos {
            new_position.x = right_most_pos - wall_bounce;
        }

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

pub fn agents_movement(
    // mut commands: Commands,
    // mut query_debug: Query<Entity, With<DebugQuad>>,
    mut game: ResMut<Game>,
    time: Res<Time>,
    mut query: Query<(&mut Transform, &AgentId), (With<NPC>, Without<Cam>)>,

    // mut cam_query: Query<&mut Transform, With<Cam>>,
    move_params: Res<MovementParams>,
) {
    // let mut rng = rand::thread_rng();
    // for (id, agent) in game.agents.iter_mut() {
    for (mut transform, agent_id) in &mut query.iter_mut() {
        let mut agent = game.agents.get_mut(&agent_id.kdtree_hash).unwrap();

        // if let Some(pos) = agent.target_position {
        let target_dir = agent.target_position - agent.position;
        if target_dir != Vec2::ZERO {
            // let target_angle = target_dir.y.atan2(target_dir.x);
            let look_at_dir = agent.compute_look_at_dir();
            let look_at_90 = Vec2::new(-look_at_dir.y, look_at_dir.x);

            let dot_dirs = look_at_90.dot(target_dir.normalize());

            // println!("look_at_dir: {:?}", dot_dirs);

            // let delta_angle = target_angle - agent.look_at_angle - std::f32::consts::PI / 2.0;

            if dot_dirs < 0.0 {
                agent.turning = Turning::Right(dot_dirs.abs());
            } else {
                agent.turning = Turning::Left(dot_dirs.abs());
            }

            agent.acc = Acceleration::Forward;
            // println!("target angle: {:?}", dot_dirs);
        }
        // }

        agent.compute_self_velocity();

        let timestep = time.delta_seconds() as f32;

        // let friction = 0.05;
        // let turning_speed_dependence = 0.03;
        // let backwards_mult = 0.3;
        // let boost_mult = 3.0;
        // let rest_turn_speed = 0.02;
        // let max_turn_speed = 0.05;
        // let throttle = 10000.0;

        let friction1 = move_params.friction1;
        let friction2 = move_params.friction2;
        let turning_speed_dependence = move_params.turning_speed_dependence;
        let backwards_mult = move_params.backwards_mult;
        let boost_mult = move_params.boost_mult;
        let rest_turn_speed = move_params.rest_turn_speed;
        let max_turn_speed = move_params.max_turn_speed;

        let mut throttle = move_params.throttle;

        if agent.is_guardian {
            throttle *= 4.0;
        }
        // let _downcurrent = move_params.downcurrent;
        let bottom_bounce = move_params.bottom_bounce;

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

        // let (left, right) = agent.compute_left_and_right_dir();

        // apply turning

        let soft_angular = 0.5;

        match agent.turning {
            Turning::Left(mut delta_angle) => {
                if delta_angle < soft_angular {
                    delta_angle = delta_angle / soft_angular;
                } else {
                    delta_angle = 1.0;
                }
                let speed_turn = agent.speed * turning_speed_dependence * (1.0 - boost_value);
                turn_angle = delta_angle
                    * (rest_turn_speed + speed_turn.clamp(0.0, max_turn_speed - rest_turn_speed));
            }
            Turning::Right(mut delta_angle) => {
                if delta_angle < soft_angular {
                    delta_angle = delta_angle / soft_angular;
                } else {
                    delta_angle = 1.0;
                }
                let speed_turn = agent.speed * turning_speed_dependence * (1.0 - boost_value);
                turn_angle =
                    -rest_turn_speed - speed_turn.clamp(0.0, max_turn_speed - rest_turn_speed);
            }
            Turning::None => {}
        }

        let friction_force = -friction1 * verlet_velocity.length() * velocity_dir
            - friction2 * verlet_velocity.length().powf(2.0) * velocity_dir;

        // no downcurrent force for agents
        let downcurrent_force = 0.0; // downcurrent * Vec2::new(0.0, -1.0);

        acc += friction_force + downcurrent_force;

        new_position += verlet_velocity + acc * timestep * timestep * throttle;

        // cannot fall below the ground
        let bottom_most_pos = agent.radius;
        if new_position.y < bottom_most_pos {
            new_position.y = bottom_most_pos + bottom_bounce;
        }

        // bounce off the walls
        // TODO: make wall_bounce part of the movement params
        let wall_bounce = 4.0;
        let left_most_pos = agent.radius;
        if new_position.x < left_most_pos {
            new_position.x = left_most_pos + wall_bounce;
        }

        let right_most_pos = LEVEL_WIDTH - agent.radius;
        if new_position.x > right_most_pos {
            new_position.x = right_most_pos - wall_bounce;
        }

        agent.speed = (new_position - agent.position).length();

        agent.last_position = agent.position;

        agent.position = new_position;

        agent.look_at_angle = (agent.look_at_angle + turn_angle) % (2.0 * std::f32::consts::PI);

        transform.translation = agent.position.extend(MAIN_CHARA_Z);

        transform.rotation = Quat::from_rotation_z(agent.look_at_angle);
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
