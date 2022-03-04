// use itertools::izip;

use bevy::prelude::*;

use rand::prelude::*;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use kdtree::distance::squared_euclidean;
use kdtree::ErrorKind;
use kdtree::KdTree;

use std::collections::HashMap;

use crate::agent::*;
use crate::*;

// use crate::{ATOM_MULT, MASS_MULT};
// use crate::vec2::*;

// use bevy::prelude::*;

pub const NUM_AGENTS: usize = 100;
pub const NUM_ITEMS: usize = 10;
pub const NUM_FOODS: usize = 5000;

pub const LEVEL_WIDTH: f32 = 5000.0;
pub const LEVEL_HEIGHT: f32 = 7000.0;

pub const MAIN_CHARA_Z: f32 = 0.1;
pub const TOTAL_BOOST_TIME: f32 = 0.3;

pub const MASS_MULT: f32 = 1000.0;

pub const ATOM_MULT: f32 = 0.15;

pub const BOTTOM_STAGE_LIMIT: f32 = 0.05;
pub const MID_STAGE_LIMIT: f32 = 0.2;
pub const TOP_STAGE_LIMIT: f32 = 0.98;

pub const BOTTOM_LIMIT_X_MIN: f32 = 0.3;
pub const BOTTOM_LIMIT_X_MAX: f32 = 0.7;

pub const MASS_EXCHANGE_RATE: f32 = 0.03;

pub const COLLISION_BOUNCE: f32 = 4.0;

pub const ENERGY_INCREASE_RATE: f32 = 0.03;

pub const ENERGY_DECAY_RATE: f32 = 0.0001;
pub const ENERGY_REGAIN_RATE: f32 = 0.01;

pub const ENERGY_GROUND_STATE: f32 = 1.0;

type TeamId = u32;

#[derive(Component)]
pub struct NPC;

#[derive(Component)]
pub struct FoodComp;

#[derive(Component)]
pub struct StartText;

pub struct CollisionEvent {
    pub agent_id: u32,
    pub other_agent_id: u32,
    pub other_is_guardian: bool,
}

#[derive(Clone, Debug)]
pub struct Team {
    pub id: TeamId,
    pub total_mass: f32,
    pub maximum_mass: f32,
    pub agents: Vec<AgentId>,
}

pub struct KdTrees {
    pub agent_kdtree: KdTree<f32, u32, [f32; 2]>,
    pub item_kdtree: KdTree<f32, u32, [f32; 2]>,
    pub food_kdtree: KdTree<f32, u32, [f32; 2]>,
}

impl KdTrees {
    pub fn new() -> Self {
        KdTrees {
            agent_kdtree: KdTree::with_capacity(2, NUM_AGENTS),
            item_kdtree: KdTree::with_capacity(2, NUM_ITEMS),
            food_kdtree: KdTree::with_capacity(2, NUM_FOODS),
        }
    }

    pub fn populate(&mut self, game: &Game) {
        self.gen_agent_kdtree(&game.agents);
        // self.gen_item_kdtree(&game.items);
        // self.gen_food_kdtree(&game.foods);
    }

    pub fn gen_agent_kdtree(&mut self, agents: &HashMap<u32, Agent>) {
        let dimensions = 2;
        // let rng = rand::thread_rng();
        let mut kdtree = KdTree::with_capacity(dimensions, NUM_AGENTS);
        agents.iter().for_each(|(id, agent)| {
            if agent.position.x.is_finite() && agent.position.y.is_finite() {
                kdtree
                    .add([agent.position.x, agent.position.y], *id)
                    .unwrap();
            } else {
                println!("infinite value skipped",);
            }
        });

        self.agent_kdtree = kdtree;
    }

    // pub fn gen_item_kdtree(&mut self, items: &HashMap<u32, Item>) {
    //     let dimensions = 2;
    //     let mut kdtree = KdTree::with_capacity(dimensions, 9);
    //     items.iter().for_each(|(id, item)| {
    //         kdtree.add([item.position.x, item.position.y], *id).unwrap();
    //     });
    //     // kdtree
    //     self.item_kdtree = kdtree;
    // }

    // pub fn gen_food_kdtree(&mut self, foods: &HashMap<u32, Food>) {
    //     let dimensions = 2;
    //     let mut kdtree = KdTree::with_capacity(dimensions, 9);
    //     foods.iter().for_each(|(id, food)| {
    //         kdtree.add([food.position.x, food.position.y], *id).unwrap();
    //     });
    //     self.food_kdtree = kdtree;
    //     // kdtree
    // }
}

pub struct Game {
    pub time: f32,
    pub game_stage: GameStage,
    pub agents: HashMap<u32, Agent>,
    // pub items: HashMap<u32, Item>,
    pub foods: HashMap<u32, Food>,

    pub teams: HashMap<TeamId, Team>,
    pub won: bool,
}

impl Game {
    pub fn new() -> Game {
        let agents = Self::gen_game_agents(NUM_AGENTS);
        // let items = Self::gen_items(NUM_ITEMS);
        let foods = Self::gen_foods(NUM_FOODS);

        // println!("generating");

        Game {
            time: 0.0,
            game_stage: GameStage::Bottom,

            agents: agents,
            // items: items,
            foods: foods,

            teams: HashMap::new(),
            won: false,
        }
    }

    // // mass of agent, foods and items increases with game stages
    // // TODO
    // pub fn gen_agents(num_agents: usize) -> HashMap<u32, Agent> {
    //     let mut rng = rand::thread_rng();
    //     let mut agents = HashMap::new();
    //     (0..num_agents).for_each(|_| {
    //         //
    //         let random_stage = GameStage::iter().choose(&mut rng).unwrap();
    //         let id: u32 = rng.gen();
    //         // avoid accidentally duplicating the main character's id
    //         if id != 1 {
    //             let random_agent = Agent::gen_random(&random_stage, id);

    //             agents.insert(id, random_agent);
    //         }
    //     });
    //     agents
    // }

    pub fn gen_game_agents(num_agents: usize) -> HashMap<u32, Agent> {
        let mut rng = rand::thread_rng();
        let mut agents = HashMap::new();
        (0..num_agents / 2).for_each(|_| {
            //
            // let random_stage = GameStage::iter().choose(&mut rng).unwrap();
            let id: u32 = rng.gen();
            // avoid accidentally duplicating the main character's id
            if id != 1 {
                let random_agent = Agent::gen_random(&GameStage::Bottom, id);

                agents.insert(id, random_agent);
            }
        });

        (0..num_agents / 2).for_each(|_| {
            //

            let id: u32 = rng.gen();
            // avoid accidentally duplicating the main character's id
            if id != 1 {
                let random_agent = Agent::gen_random(&GameStage::Mid, id);

                agents.insert(id, random_agent);
            }
        });

        // guardians have ids between 20 and 40
        (0..5).for_each(|k| {
            //

            // let id: u32 = rng.gen();
            let x_pos = LEVEL_WIDTH / 2.0 * k as f32 / 10.0 * 0.85;
            let pos = Vec2::new(x_pos, LEVEL_HEIGHT - 2000.0);
            // avoid accidentally duplicating the main character's id

            let random_agent = Agent::gen_guardian(pos, k + 20);

            agents.insert(k + 20, random_agent);
        });

        (5..10).for_each(|k| {
            //

            // let id: u32 = rng.gen();
            let x_pos = LEVEL_WIDTH - LEVEL_WIDTH * (k - 5) as f32 / 10.0 * 0.85;
            let pos = Vec2::new(x_pos, LEVEL_HEIGHT - 2000.0);
            // avoid accidentally duplicating the main character's id

            let random_agent = Agent::gen_guardian(pos, k + 20);

            agents.insert(k + 20, random_agent);
        });

        (0..10).for_each(|k| {
            //

            // let id: u32 = rng.gen();
            let x_pos = LEVEL_WIDTH * k as f32 / 10.0;
            let pos = Vec2::new(x_pos, LEVEL_HEIGHT - 4000.0);
            // avoid accidentally duplicating the main character's id

            let random_agent = Agent::gen_guardian(pos, k + 30);

            agents.insert(k + 30, random_agent);
        });

        // (0..10).for_each(|k| {
        //     //

        //     let id: u32 = rng.gen();
        //     let x_pos = LEVEL_WIDTH * k as f32 / 10.0;
        //     let pos = Vec2::new(x_pos, LEVEL_HEIGHT - 4000.0);
        //     // avoid accidentally duplicating the main character's id
        //     if id != 1 {
        //         let random_agent = Agent::gen_guardian(pos, id);

        //         agents.insert(id, random_agent);
        //     }
        // });
        agents
    }

    // // TODO
    // pub fn gen_items(num_items: usize) -> HashMap<u32, Item> {
    //     let mut rng = rand::thread_rng();
    //     let mut items = HashMap::new();

    //     (0..num_items).for_each(|_| {
    //         let random_stage = GameStage::iter().choose(&mut rng).unwrap();
    //         let id: u32 = rng.gen();
    //         let random_item = Item::random_item(random_stage, id);

    //         items.insert(id, random_item);
    //     });

    //     items
    // }

    // TODO
    pub fn gen_foods(num_foods: usize) -> HashMap<u32, Food> {
        let mut rng = rand::thread_rng();
        let mut foods = HashMap::new();

        // let id: u32 = ;

        (0..num_foods).for_each(|_| {
            let y: f32 = rng.gen_range(0.0..1.0) * LEVEL_HEIGHT;
            let x: f32 = rng.gen_range(0.0..1.0) * LEVEL_WIDTH;

            foods.insert(
                rng.gen::<u32>(),
                Food {
                    position: Vec2::new(x, y),
                    energy: rng.gen_range(0.0..0.02),
                    mass: rng.gen_range(0.0..0.02),
                    id: rng.gen::<u32>(),
                    acc: Vec2::new(0.0, 0.0),
                },
            );

            // let stage = GameStage::iter().choose(&mut rng).unwrap();
            // match stage {
            //     GameStage::Bottom => {
            //         let y: f32 = rng.gen_range(0.0..BOTTOM_STAGE_LIMIT) * LEVEL_HEIGHT;
            //         let x: f32 =
            //             rng.gen_range(BOTTOM_LIMIT_X_MIN..BOTTOM_LIMIT_X_MAX) * LEVEL_WIDTH;

            //         foods.insert(
            //             rng.gen::<u32>(),
            //             Food {
            //                 position: Vec2::new(x, y),
            //                 energy: rng.gen_range(0.0..0.02),
            //                 mass: rng.gen_range(0.0..0.02),
            //                 id: rng.gen::<u32>(),
            //                 acc: Vec2::new(0.0, 0.0),
            //             },
            //         );
            //     }

            //     GameStage::Mid => {
            //         let y: f32 = rng.gen_range(BOTTOM_STAGE_LIMIT..MID_STAGE_LIMIT) * LEVEL_HEIGHT;
            //         let x: f32 = rng.gen_range(0.0..1.0) * LEVEL_WIDTH;

            //         foods.insert(
            //             rng.gen::<u32>(),
            //             Food {
            //                 position: Vec2::new(x, y),
            //                 energy: rng.gen_range(BOTTOM_STAGE_LIMIT..MID_STAGE_LIMIT),
            //                 mass: rng.gen_range(BOTTOM_STAGE_LIMIT..MID_STAGE_LIMIT),
            //                 id: rng.gen::<u32>(),
            //                 acc: Vec2::new(0.0, 0.0),
            //             },
            //         );
            //     }

            //     GameStage::Top => {
            //         let y: f32 = rng.gen_range(MID_STAGE_LIMIT..TOP_STAGE_LIMIT) * LEVEL_HEIGHT;
            //         let x: f32 = rng.gen_range(0.0..TOP_STAGE_LIMIT) * LEVEL_WIDTH;

            //         foods.insert(
            //             rng.gen::<u32>(),
            //             Food {
            //                 position: Vec2::new(x, y),
            //                 energy: rng.gen_range(MID_STAGE_LIMIT..TOP_STAGE_LIMIT),
            //                 mass: rng.gen_range(MID_STAGE_LIMIT..TOP_STAGE_LIMIT),
            //                 id: rng.gen::<u32>(),
            //                 acc: Vec2::new(0.0, 0.0),
            //             },
            //         );
            //     } // _ => {}
            // }
        });

        foods
    }

    pub fn update_agent_kdtree(&self, mut agent_kdtree: &mut KdTree<f32, u32, [f32; 2]>) {
        let dimensions = 2;
        let mut kdtree = KdTree::with_capacity(dimensions, NUM_AGENTS);
        self.agents.iter().for_each(|(id, agent)| {
            kdtree
                .add([agent.position.x, agent.position.y], *id)
                .unwrap();
        });

        *agent_kdtree = kdtree;
    }
}

// #[derive(Clone, Debug, PartialEq)]
// pub struct Item {
//     pub id: u32,
//     pub item_type: ItemType,
//     pub position: Vec2,
//     pub mass: f32,
//     pub range: f32,
//     pub damage: f32,
//     pub hp: f32,
// }

// impl Item {
//     pub fn random_item(stage: GameStage, id: u32) -> Item {
//         let mut rng = rand::thread_rng();
//         let item_type = ItemType::random_item();

//         let position: Vec2;
//         let mass: f32;
//         let range: f32;
//         let damage: f32;
//         let hp: f32;

//         match stage {
//             GameStage::Bottom => {
//                 position = Vec2::new(
//                     rng.gen_range(BOTTOM_LIMIT_X_MIN..BOTTOM_LIMIT_X_MAX),
//                     rng.gen_range(0.0..BOTTOM_STAGE_LIMIT),
//                 );
//                 mass = rng.gen_range(0.03..BOTTOM_STAGE_LIMIT);
//                 range = rng.gen_range(0.0..BOTTOM_STAGE_LIMIT);
//                 damage = rng.gen_range(0.0..BOTTOM_STAGE_LIMIT);
//                 hp = rng.gen_range(0.0..BOTTOM_STAGE_LIMIT);
//             }
//             GameStage::Mid => {
//                 position = Vec2::new(
//                     rng.gen_range(0.0..1.0),
//                     rng.gen_range(BOTTOM_STAGE_LIMIT..MID_STAGE_LIMIT),
//                 );
//                 mass = rng.gen_range(BOTTOM_STAGE_LIMIT..MID_STAGE_LIMIT);
//                 range = rng.gen_range(BOTTOM_STAGE_LIMIT..MID_STAGE_LIMIT);
//                 damage = rng.gen_range(BOTTOM_STAGE_LIMIT..MID_STAGE_LIMIT);
//                 hp = rng.gen_range(BOTTOM_STAGE_LIMIT..MID_STAGE_LIMIT);
//             }
//             GameStage::Top => {
//                 position = Vec2::new(
//                     rng.gen_range(0.0..TOP_STAGE_LIMIT),
//                     rng.gen_range(MID_STAGE_LIMIT..TOP_STAGE_LIMIT),
//                 );
//                 mass = rng.gen_range(MID_STAGE_LIMIT..TOP_STAGE_LIMIT);
//                 range = rng.gen_range(MID_STAGE_LIMIT..TOP_STAGE_LIMIT);
//                 damage = rng.gen_range(MID_STAGE_LIMIT..TOP_STAGE_LIMIT);
//                 hp = rng.gen_range(MID_STAGE_LIMIT..TOP_STAGE_LIMIT);
//             }
//         };

//         return Self {
//             id,
//             item_type,
//             position,
//             mass,
//             range,
//             damage,
//             hp,
//         };

//         // println!("item: {:?}", item);
//         // return item;
//     }
// }

#[derive(Clone, Debug, Copy, EnumIter, PartialEq)]
pub enum ItemType {
    Propeller,
    FoodVacuum,
    CreatureVacuum,
    Weapon,
    Sonar,
}

impl ItemType {
    pub fn random_item() -> ItemType {
        let mut rng = rand::thread_rng();
        let rand_int = rng.gen_range(0..ItemType::iter().count());
        let item_type = ItemType::iter().nth(rand_int).unwrap();
        item_type
    }
}

#[derive(Clone, Debug, PartialEq, EnumIter)]
pub enum GameStage {
    Bottom,
    Mid,
    Top,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Food {
    pub position: Vec2,
    pub energy: f32,
    pub mass: f32,
    pub id: u32,
    pub acc: Vec2,
}

#[derive(Clone, Debug)]
pub enum Direction {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

#[derive(Component)]
pub struct MainCharacter {
    pub id: u32,
}

#[derive(Component)]
pub struct DebugQuad;

pub fn see(
    mut game: ResMut<Game>,
    kdtrees: ResMut<KdTrees>,
    time: Res<Time>,
    mut commands: Commands,
    query_debug: Query<Entity, With<DebugQuad>>,
) {
    let all_agents = game.agents.clone();
    // let all_items = game.items.clone();
    let all_foods = game.foods.clone();

    // let main_agent = all_agents.get(&1).unwrap();
    // let sights = main_agent.sensors.agent_sight.clone();
    for debug_quad in query_debug.iter() {
        commands.entity(debug_quad).despawn();
    }

    for (hash_id, mut agent) in &mut game.agents {
        // checks if agent is close enough to be seen

        // println!(
        //     "mass: {}, range: {:?}",
        //     agent.mass,
        //     (agent.sensors.sight_range)
        // );
        if let Ok(dist_id_array) = kdtrees.agent_kdtree.nearest(
            &[agent.position.x, agent.position.y],
            // (agent.sensors.sight_range).powf(2.0),
            3,
            // (100.0_f32).powf(2.0),
            &squared_euclidean,
        ) {
            //
            // agent.sensors.sight.

            for (dist, id) in dist_id_array {
                // the kdtree contains the agent seeing itself, so we need to skip it.
                // No consciousness allowed in this game!
                if hash_id == id {
                    continue;
                }

                let other_agent = all_agents.get(&id).unwrap();
                // let other_agent_pos = other_agent.position;
                let direction_to_other_agent = other_agent.position - agent.position;

                // let dir_to_other_angle = direction_to_other_agent.y.atan2(direction_to_other_agent.x);
                let agent_looking_direction =
                    Vec2::new(agent.look_at_angle.cos(), agent.look_at_angle.sin());
                // if the angles differ by more than 180 degrees, then the other agent is on the other side of the agent
                // if agent_looking_direction.dot(direction_to_other_agent) > 0.0 {
                agent.update_agent_sight(
                    time.seconds_since_startup() as f32,
                    dist.sqrt(),
                    other_agent,
                );

                // despawn and spawn green cube upon seeing
                // if *hash_id == 1 {
                // println!("yup");
                // println!("yup; {:?}", agent.sensors.agent_sight.len());

                // for (other_id, other_agent) in agent.sensors.agent_sight.clone() {
                //     let mut rng = rand::thread_rng();
                //     commands
                //         .spawn_bundle(SpriteBundle {
                //             sprite: Sprite {
                //                 color: Color::rgb(1.0, 0.0, 0.0),
                //                 custom_size: Some(Vec2::splat(10.0)),

                //                 ..Default::default()
                //             },
                //             transform: Transform::from_translation(
                //                 (other_agent.position).extend(2.09),
                //             ),
                //             ..Default::default()
                //         })
                //         .insert(DebugQuad);
                // }
            }

            // if let Ok(dist_id_array) = kdtrees.item_kdtree.within(
            //     &[agent.position.x, agent.position.y],
            //     agent.sensors.sight_range,
            //     &squared_euclidean,
            // ) {
            //     //
            //     // agent.sensors.sight.

            //     for (dist, id) in dist_id_array {
            //         let item = all_items.get(&id).unwrap();
            //         // let other_agent_pos = other_agent.position;
            //         let direction_to_item = item.position - agent.position;

            //         // let dir_to_other_angle = direction_to_other_agent.y.atan2(direction_to_other_agent.x);
            //         let agent_looking_direction =
            //             Vec2::new(agent.look_at_angle.cos(), agent.look_at_angle.sin());
            //         // if the angles differ by more than 180 degrees, then the other agent is on the other side of the agent
            //         // if agent_looking_direction.dot(direction_to_item) > 0.0 {
            //         agent.update_item_sight(time.seconds_since_startup() as f32, dist, item);

            //         // despawn and spawn green cube upon seeing
            //         // }
            //     }
            // }

            // if let Ok(dist_id_array) = kdtrees.food_kdtree.within(
            //     &[agent.position.x, agent.position.y],
            //     agent.sensors.sight_range,
            //     &squared_euclidean,
            // ) {
            //     //
            //     // agent.sensors.sight.

            //     for (dist, id) in dist_id_array {
            //         let food = all_foods.get(&id).unwrap();
            //         // let other_agent_pos = other_agent.position;
            //         let direction_to_food = food.position - agent.position;

            //         // let dir_to_other_angle = direction_to_other_agent.y.atan2(direction_to_other_agent.x);
            //         let agent_looking_direction =
            //             Vec2::new(agent.look_at_angle.cos(), agent.look_at_angle.sin());
            //         // if the angles differ by more than 180 degrees, then the other agent is on the other side of the agent
            //         // if agent_looking_direction.dot(direction_to_food) > 0.0 {
            //         agent.update_food_sight(time.seconds_since_startup() as f32, dist, food);

            //         // despawn and spawn green cube upon seeing
            //         // }
            //     }
            // }
        }
    }
}

pub fn forget(mut game: ResMut<Game>, kdtrees: ResMut<KdTrees>, time: Res<Time>) {
    let mut rng = thread_rng();

    for (_hash_id, mut agent) in &mut game.agents {
        // run once every ten frames on average
        if rng.gen::<f32>() < 0.1 {
            agent.forget_agents(time.seconds_since_startup() as f32);
            // agent.forget_items(time.seconds_since_startup() as f32);
            // agent.forget_food(time.seconds_since_startup() as f32);
        }
    }
}

pub fn sigmoid(x: f32, sign: f32, up: f32, lo: f32, slope: f32, attr: f32) -> f32 {
    lo + (up - lo) * ((1.0 - sign) / 2.0 + sign / (1.0 + slope * (x - 1.0 + attr).exp()))
}

pub struct PosMass {
    pub position: Vec2,
    pub mass: f32,
}

pub fn compute_acc_food(mut game: ResMut<Game>, kdtrees: Res<KdTrees>) {
    // compute forces on food
    let mut pos_mass: Vec<PosMass> = game
        .agents
        .iter()
        .map(|(_, agent)| PosMass {
            position: agent.position,
            mass: agent.mass,
        })
        .collect();

    let food_kd_tree = &kdtrees.food_kdtree;

    pos_mass.iter().for_each(|pos_mass| {
        let pos = pos_mass.position;
        // TODO: tweak this
        let radius = pos_mass.mass;
        if let Ok(dist_id_array) = food_kd_tree.within(&[pos.x, pos.y], radius, &squared_euclidean)
        {
            //
            dist_id_array.iter().for_each(|(dist, id)| {
                let food = game.foods.get_mut(&id).unwrap();
                let force_direction = (pos - food.position).normalize();

                // TODO: tweak this
                let force_amplitude = 1.0 / (dist + 0.01) * pos_mass.mass / food.mass;
                food.acc = food.acc + force_direction * force_amplitude;
            });
        }
    });
}

pub fn eat_food(mut game: ResMut<Game>, kdtrees: Res<KdTrees>) {
    let mut foods_to_remove: Vec<u32> = Vec::new();

    foods_to_remove.iter().for_each(|id| {
        game.foods.remove(id);

        // TODO: despawn food
    });
}

pub fn load_character_auto(
    keyboard: Res<Input<KeyCode>>,
    mut query: Query<&mut MarkerInstanceMatData>,
) {
    if keyboard.pressed(KeyCode::LControl) && keyboard.just_pressed(KeyCode::L) {
        let contents = include_str!("bah.cha");

        let loaded_character: CharacterSaveFormat = serde_json::from_str(&contents).unwrap();

        for mut character in query.iter_mut() {
            let character2 = character.as_mut();
            *character2 = loaded_character.clone().into();
        }
    }
}

pub fn update_agent_kdtree(mut game: ResMut<Game>, mut kdtrees: ResMut<KdTrees>) {
    game.update_agent_kdtree(&mut kdtrees.agent_kdtree);
}

#[derive(Debug)]
pub struct AgentCollisionInfo {
    pub agent_id1: u32,
    pub atom_entity1: Entity,
    // pub acceleration1: Vec2,
    pub other_collision_mass1: f32,
    pub velocity1: Vec2,
    pub is_guardian1: bool,

    pub agent_id2: u32,
    pub atom_entity2: Entity,
    // pub acceleration2: Vec2,
    pub velocity2: Vec2,
    pub other_collision_mass2: f32,
    pub is_guardian2: bool,
}

pub fn collisions(
    mut game: ResMut<Game>,
    mut kdtrees: ResMut<KdTrees>,
    mut sprite_query: Query<&mut Sprite, With<Atom>>,
    atoms_query: Query<&Children, With<AgentId>>,
    atom_transform_query: Query<&GlobalTransform, With<Atom>>,
    mut collision_event: EventWriter<CollisionEvent>,
) {
    let agent_bodies = game
        .agents
        .iter()
        .map(|(id, x)| (*id, x.body.clone()))
        .collect::<HashMap<_, _>>();

    let mut collisions: Vec<AgentCollisionInfo> = Vec::new();

    for mut sprite in sprite_query.iter_mut() {
        sprite.color = Color::GREEN;
    }

    let mut collisioned_agents = Vec::new();

    for (id, agent) in game.agents.iter() {
        if collisioned_agents.contains(&id) {
            continue;
        }

        if let Ok(closest_agents) = kdtrees.agent_kdtree.nearest(
            &[agent.position.x, agent.position.y],
            2,
            &squared_euclidean,
        ) {
            // the closest is itself, so we ignore the first item
            let closest_agent_id = closest_agents[1].1;

            if collisioned_agents.contains(&closest_agent_id) {
                continue;
            }

            let closest_agent_dist = closest_agents[1].0;

            let closest_agent = game.agents.get(&closest_agent_id).unwrap();

            let mut agent_pair_checked = Vec::new();

            // for (other_id, closest_agent) in agent.sensors.agent_sight.iter() {
            // perhaps a collision

            // nodes are at a max distance of 0.5 * mass * MASS_MULT from the center of the quad
            let distance_test = (closest_agent.mass + agent.mass) * MASS_MULT * 0.5;

            // println!("closest: {}", closest_agent_dist);

            // squared euclidian
            if closest_agent_dist < distance_test * distance_test {
                // 1.00 = buffer distance?

                if !agent_pair_checked.contains(&(closest_agent.id, agent.id)) {
                    agent_pair_checked.push((closest_agent.id, agent.id));
                    agent_pair_checked.push((agent.id, closest_agent.id));

                    // let body = &agent.body;
                    // let other_body = agent_bodies.get(&closest_agent.id).unwrap();

                    // for (j, atom) in body.iter().filter(|b| b.is_used).enumerate() {
                    //     let mut found_collision = false;

                    //     for (k, other_atom) in other_body.iter().filter(|b| b.is_used).enumerate() {

                    let atoms1 = atoms_query.get(agent.entity.unwrap()).unwrap();
                    let atoms2 = atoms_query.get(closest_agent.entity.unwrap()).unwrap();

                    for child1 in atoms1.iter() {
                        let mut found_collision = false;
                        let atom_transform1 = atom_transform_query.get(*child1).unwrap();

                        for child2 in atoms2.iter() {
                            //

                            let atom_transform2 = atom_transform_query.get(*child2).unwrap();

                            let global_atom_pos = atom_transform1.translation.truncate();
                            let other_global_atom_pos = atom_transform2.translation.truncate();

                            let dist = (global_atom_pos - other_global_atom_pos).length();

                            if dist < (agent.radius + closest_agent.radius) {
                                // if dist < (agent.radius + closest_agent.radius) / 10.0 {
                                // keep track of the agents that have collided, so that we don't compute more
                                // collision for them during this frame
                                collisioned_agents.push(&closest_agent.id);
                                collisioned_agents.push(&agent.id);

                                let u1 = agent.velocity;
                                let u2 = Vec2::new(
                                    closest_agent.look_at_angle.cos() * closest_agent.speed,
                                    closest_agent.look_at_angle.sin() * closest_agent.speed,
                                );

                                let m_ratio1 =
                                    2.0 * closest_agent.mass / (closest_agent.mass + agent.mass);
                                let m_ratio2 = 2.0 * agent.mass / (closest_agent.mass + agent.mass);

                                let mut collision_line = closest_agent.position - agent.position;
                                if collision_line != Vec2::ZERO {
                                    collision_line = collision_line.normalize();
                                } else {
                                    collision_line = Vec2::new(1.0, 0.0);
                                }

                                let velocity1 = -collision_line * m_ratio1 * 4.0;
                                let velocity2 = collision_line * m_ratio2 * 4.0;

                                let other_collision_mass1 = closest_agent.mass;
                                let other_collision_mass2 = agent.mass;

                                let collision = AgentCollisionInfo {
                                    agent_id1: agent.id,
                                    atom_entity1: *child1,
                                    other_collision_mass1,
                                    is_guardian1: agent.is_guardian,
                                    // velocity1: v1,
                                    velocity1,
                                    agent_id2: closest_agent.id,
                                    atom_entity2: child2.clone(),
                                    other_collision_mass2,
                                    velocity2,
                                    is_guardian2: closest_agent.is_guardian,
                                    // velocity2: v2,
                                };
                                // println!("collision: {:?}", collision);

                                collisions.push(collision);

                                found_collision = true;
                                // println!("collision: {}", dist);

                                // let mut sprite1 = sprite_query.get_mut(atom.entity.unwrap()).unwrap();
                                // sprite1.color = Color::RED;

                                // let mut sprite2 =
                                //     sprite_query.get_mut(other_atom.entity.unwrap()).unwrap();
                                // sprite2.color = Color::RED;

                                let mut sprite1 = sprite_query.get_mut(*child1).unwrap();
                                sprite1.color = Color::RED;

                                let mut sprite2 = sprite_query.get_mut(*child2).unwrap();
                                sprite2.color = Color::RED;

                                break;
                            }
                            if found_collision {
                                break;
                            }
                        }
                        if found_collision {
                            break;
                        }
                    }
                    // }
                }
            }
        }
    }

    let mut id_cache = Vec::new();
    for collision in collisions {
        /////////////// agent 1 /////////////////////////////////////////////////////////
        let mut agent = game.agents.get_mut(&collision.agent_id1).unwrap();

        id_cache.push(agent.id);

        // here no properties are changed, just information about the collision
        agent.last_position = agent.position - collision.velocity1 * COLLISION_BOUNCE;
        agent.just_collided = true;
        agent.other_collider_mass = collision.other_collision_mass1;

        collision_event.send(CollisionEvent {
            agent_id: collision.agent_id1,
            other_agent_id: collision.agent_id2,
            other_is_guardian: collision.is_guardian2,
        });

        /////////////// agent 2 /////////////////////////////////////////////////////////

        let mut closest_agent = game.agents.get_mut(&collision.agent_id2).unwrap();

        id_cache.push(closest_agent.id);

        closest_agent.last_position = closest_agent.position - collision.velocity2;
        closest_agent.just_collided = true;
        closest_agent.other_collider_mass = collision.other_collision_mass2;

        collision_event.send(CollisionEvent {
            agent_id: collision.agent_id2,
            other_agent_id: collision.agent_id1,
            other_is_guardian: collision.is_guardian1,
        });
    }
}

pub fn agent_decisions(mut game: ResMut<Game>, time: Res<Time>) {
    let mut rng = rand::thread_rng();

    for (id, agent) in game.agents.iter_mut() {
        if *id == 1 {
            // println!("sightings: {:?}", agent.sensors.agent_sight);
        }
        // make a decision once per 10 frames on average

        // if the past goal has been going on for too long, change it
        if time.seconds_since_startup() as f32 - agent.goal_time > agent.memory_time {
            for (seen_agent_id, _agent_sighting) in agent.sensors.agent_sight.iter() {
                //
                if agent.is_guardian {
                    if seen_agent_id == &1 {
                        agent.goal = Goal::Bully(seen_agent_id.clone());
                        agent.goal_time = time.seconds_since_startup() as f32;
                        break;
                    }
                }

                if rng.gen::<f32>() < 0.1 {
                    if *seen_agent_id != agent.last_agent_hit {
                        if rng.gen::<f32>() < 0.2 {
                            agent.goal = Goal::Bully(seen_agent_id.clone());
                            agent.goal_time = time.seconds_since_startup() as f32;
                            break;
                        }
                    }
                }
            }
        }
    }
}

// increase energy if the last agent hit isn't the same as the previous one
pub fn update_agent_properties(
    mut game: ResMut<Game>,
    time: Res<Time>,
    // mut agents_transform_query: Query<&mut Transform, With<AgentId>>,
    // atoms_query: Query<&Children, With<AgentId>>,
    // mut atom_transform_query: Query<&mut Transform, (With<Atom>, Without<AgentId>)>,
    mut collision_event: EventReader<CollisionEvent>,
) {
    for collision_info in collision_event.iter() {
        // let other_agent = game.agents.get(&collision_info.other_agent_id).unwrap();
        // let is_guardian = other_agent.is_guardian;
        let mut agent = game.agents.get_mut(&collision_info.agent_id).unwrap();
        // unused
        if collision_info.other_is_guardian && agent.id == 1 {
            agent.energy *= 0.5;
            println!("energy GUARDIAN SMASH: {}", agent.energy);
        }

        if agent.last_agent_hit != collision_info.other_agent_id {
            // if agent.id == 1 {
            //     println!("increasing energy",);
            // }
            agent.energy *= 1.0 + ENERGY_INCREASE_RATE;
            agent.last_agent_hit = collision_info.other_agent_id;
            if agent.id == 1 {
                println!("energy increase {}", agent.energy);
            }
        } else {
            // if agent.id == 1 {
            //     println!("decreasing energy",);
            // }
            agent.energy *= 1.0 - ENERGY_INCREASE_RATE;
            agent.last_agent_hit = collision_info.other_agent_id;
            if agent.id == 1 {
                println!("energy decrease {}", agent.energy);
            }
        }
        agent.last_collision_time = time.seconds_since_startup() as f32;
    }
}

pub fn agent_action(mut game: ResMut<Game>) {
    let mut rng = rand::thread_rng();

    let agent_positions = game
        .agents
        .iter()
        .map(|(id, x)| (*id, x.position))
        .collect::<HashMap<_, _>>();

    for (_id, agent) in game.agents.iter_mut() {
        // if rng.gen::<f32>() < 0.5 {
        agent.act(&agent_positions);
        // }
    }
}
