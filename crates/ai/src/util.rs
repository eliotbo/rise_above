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
use crate::libaaa::*;

// use crate::{ATOM_MULT, MASS_MULT};
// use crate::vec2::*;

// use bevy::prelude::*;

pub const NUM_AGENTS: usize = 100;
pub const NUM_ITEMS: usize = 10;
pub const NUM_FOODS: usize = 2000;

pub const LEVEL_WIDTH: f32 = 30000.0;
pub const LEVEL_HEIGHT: f32 = 20000.0;

// pub const POS_MULT: f32 = 10000.0;

// pub const BOOST_TIMER: f32 = 0.3;

pub const MAIN_CHARA_Z: f32 = 0.1;
pub const TOTAL_BOOST_TIME: f32 = 0.3;

pub const MASS_MULT: f32 = 1000.0;

pub const ATOM_MULT: f32 = 0.05;

pub const BOTTOM_STAGE_LIMIT: f32 = 0.05;
pub const MID_STAGE_LIMIT: f32 = 0.2;
pub const TOP_STAGE_LIMIT: f32 = 0.95;

pub const BOTTOM_LIMIT_X_MIN: f32 = 0.45;
pub const BOTTOM_LIMIT_X_MAX: f32 = 0.55;

pub const MASS_EXCHANGE_RATE: f32 = 0.03;

type TeamId = u32;

#[derive(Component)]
pub struct NPC;

#[derive(Component)]
pub struct FoodComp;

pub struct CollisionEvent {
    pub agent_id: u32,
    pub previous_mass: f32,
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
        self.gen_item_kdtree(&game.items);
        self.gen_food_kdtree(&game.foods);
    }

    pub fn gen_agent_kdtree(&mut self, agents: &HashMap<u32, Agent>) {
        let dimensions = 2;
        let mut kdtree = KdTree::with_capacity(dimensions, NUM_AGENTS);
        agents.iter().for_each(|(id, agent)| {
            kdtree
                .add([agent.position.x, agent.position.y], *id)
                .unwrap();
        });

        self.agent_kdtree = kdtree;
    }

    pub fn gen_item_kdtree(&mut self, items: &HashMap<u32, Item>) {
        let dimensions = 2;
        let mut kdtree = KdTree::with_capacity(dimensions, 9);
        items.iter().for_each(|(id, item)| {
            kdtree.add([item.position.x, item.position.y], *id).unwrap();
        });
        // kdtree
        self.item_kdtree = kdtree;
    }

    pub fn gen_food_kdtree(&mut self, foods: &HashMap<u32, Food>) {
        let dimensions = 2;
        let mut kdtree = KdTree::with_capacity(dimensions, 9);
        foods.iter().for_each(|(id, food)| {
            kdtree.add([food.position.x, food.position.y], *id).unwrap();
        });
        self.food_kdtree = kdtree;
        // kdtree
    }
}

pub struct Game {
    pub time: f32,
    pub game_stage: GameStage,
    pub agents: HashMap<u32, Agent>,
    pub items: HashMap<u32, Item>,
    pub foods: HashMap<u32, Food>,

    pub teams: HashMap<TeamId, Team>,
}

impl Game {
    pub fn new() -> Game {
        let agents = Self::gen_agents(NUM_AGENTS);
        let items = Self::gen_items(NUM_ITEMS);
        let foods = Self::gen_foods(NUM_FOODS);

        // println!("generating");

        Game {
            time: 0.0,
            game_stage: GameStage::Bottom,

            agents: agents,
            items: items,
            foods: foods,

            teams: HashMap::new(),
        }
    }

    // mass of agent, foods and items increases with game stages
    // TODO
    pub fn gen_agents(num_agents: usize) -> HashMap<u32, Agent> {
        let mut rng = rand::thread_rng();
        let mut agents = HashMap::new();
        (0..num_agents).for_each(|_| {
            //
            let random_stage = GameStage::iter().choose(&mut rng).unwrap();
            let id: u32 = rng.gen();
            // avoid accidentally duplicating the main character's id
            if id != 1 {
                let random_agent = Agent::gen_random(&random_stage, id);

                agents.insert(id, random_agent);
            }
        });
        agents
    }

    // TODO
    pub fn gen_items(num_items: usize) -> HashMap<u32, Item> {
        let mut rng = rand::thread_rng();
        let mut items = HashMap::new();

        (0..num_items).for_each(|_| {
            let random_stage = GameStage::iter().choose(&mut rng).unwrap();
            let id: u32 = rng.gen();
            let random_item = Item::random_item(random_stage, id);

            items.insert(id, random_item);
        });

        items
    }

    // TODO
    pub fn gen_foods(num_foods: usize) -> HashMap<u32, Food> {
        let mut rng = rand::thread_rng();
        let mut foods = HashMap::new();

        // let id: u32 = ;

        (0..num_foods).for_each(|_| {
            // let stage: usize = rng.gen_range(0..3);
            let stage = GameStage::iter().choose(&mut rng).unwrap();
            match stage {
                GameStage::Bottom => {
                    let y: f32 = rng.gen_range(0.0..BOTTOM_STAGE_LIMIT) * LEVEL_HEIGHT;
                    let x: f32 =
                        rng.gen_range(BOTTOM_LIMIT_X_MIN..BOTTOM_LIMIT_X_MAX) * LEVEL_WIDTH;

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
                }

                GameStage::Mid => {
                    let y: f32 = rng.gen_range(BOTTOM_STAGE_LIMIT..MID_STAGE_LIMIT) * LEVEL_HEIGHT;
                    let x: f32 = rng.gen_range(0.0..1.0) * LEVEL_WIDTH;

                    foods.insert(
                        rng.gen::<u32>(),
                        Food {
                            position: Vec2::new(x, y),
                            energy: rng.gen_range(BOTTOM_STAGE_LIMIT..MID_STAGE_LIMIT),
                            mass: rng.gen_range(BOTTOM_STAGE_LIMIT..MID_STAGE_LIMIT),
                            id: rng.gen::<u32>(),
                            acc: Vec2::new(0.0, 0.0),
                        },
                    );
                }

                GameStage::Top => {
                    let y: f32 = rng.gen_range(MID_STAGE_LIMIT..TOP_STAGE_LIMIT) * LEVEL_HEIGHT;
                    let x: f32 = rng.gen_range(0.0..TOP_STAGE_LIMIT) * LEVEL_WIDTH;

                    foods.insert(
                        rng.gen::<u32>(),
                        Food {
                            position: Vec2::new(x, y),
                            energy: rng.gen_range(MID_STAGE_LIMIT..TOP_STAGE_LIMIT),
                            mass: rng.gen_range(MID_STAGE_LIMIT..TOP_STAGE_LIMIT),
                            id: rng.gen::<u32>(),
                            acc: Vec2::new(0.0, 0.0),
                        },
                    );
                } // _ => {}
            }
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

#[derive(Clone, Debug, PartialEq)]
pub struct Item {
    pub id: u32,
    pub item_type: ItemType,
    pub position: Vec2,
    pub mass: f32,
    pub range: f32,
    pub damage: f32,
    pub hp: f32,
}

impl Item {
    pub fn random_item(stage: GameStage, id: u32) -> Item {
        let mut rng = rand::thread_rng();
        let item_type = ItemType::random_item();

        let position: Vec2;
        let mass: f32;
        let range: f32;
        let damage: f32;
        let hp: f32;

        match stage {
            GameStage::Bottom => {
                position = Vec2::new(
                    rng.gen_range(BOTTOM_LIMIT_X_MIN..BOTTOM_LIMIT_X_MAX),
                    rng.gen_range(0.0..BOTTOM_STAGE_LIMIT),
                );
                mass = rng.gen_range(0.03..BOTTOM_STAGE_LIMIT);
                range = rng.gen_range(0.0..BOTTOM_STAGE_LIMIT);
                damage = rng.gen_range(0.0..BOTTOM_STAGE_LIMIT);
                hp = rng.gen_range(0.0..BOTTOM_STAGE_LIMIT);
            }
            GameStage::Mid => {
                position = Vec2::new(
                    rng.gen_range(0.0..1.0),
                    rng.gen_range(BOTTOM_STAGE_LIMIT..MID_STAGE_LIMIT),
                );
                mass = rng.gen_range(BOTTOM_STAGE_LIMIT..MID_STAGE_LIMIT);
                range = rng.gen_range(BOTTOM_STAGE_LIMIT..MID_STAGE_LIMIT);
                damage = rng.gen_range(BOTTOM_STAGE_LIMIT..MID_STAGE_LIMIT);
                hp = rng.gen_range(BOTTOM_STAGE_LIMIT..MID_STAGE_LIMIT);
            }
            GameStage::Top => {
                position = Vec2::new(
                    rng.gen_range(0.0..TOP_STAGE_LIMIT),
                    rng.gen_range(MID_STAGE_LIMIT..TOP_STAGE_LIMIT),
                );
                mass = rng.gen_range(MID_STAGE_LIMIT..TOP_STAGE_LIMIT);
                range = rng.gen_range(MID_STAGE_LIMIT..TOP_STAGE_LIMIT);
                damage = rng.gen_range(MID_STAGE_LIMIT..TOP_STAGE_LIMIT);
                hp = rng.gen_range(MID_STAGE_LIMIT..TOP_STAGE_LIMIT);
            }
        };

        return Self {
            id,
            item_type,
            position,
            mass,
            range,
            damage,
            hp,
        };

        // println!("item: {:?}", item);
        // return item;
    }
}

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
    let all_items = game.items.clone();
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
        if let Ok(dist_id_array) = kdtrees.agent_kdtree.within(
            &[agent.position.x, agent.position.y],
            (agent.sensors.sight_range).powf(2.0),
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

    let mut food_vacuum_pos_and_mass: Vec<PosMass> = game
        .items
        .iter()
        .filter(|(_, item)| item.item_type == ItemType::FoodVacuum)
        .map(|(_, food_vacuum)| PosMass {
            position: food_vacuum.position,
            mass: food_vacuum.mass,
        })
        .collect();

    pos_mass.append(&mut food_vacuum_pos_and_mass);

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

    // game.agents.iter_mut().for_each(|(_, mut agent)| {
    //     //
    //     // TODO: tweak this
    //     let agent_food_radius = agent.mass * 0.5;
    //     if let Ok(dist_id_array) = kdtrees.food_kdtree.within(
    //         &[agent.position.x, agent.position.y],
    //         agent_food_radius,
    //         &squared_euclidean,
    //     ) {
    //         //
    //         dist_id_array.iter().for_each(|(_dist, id)| {
    //             foods_to_remove.push(**id);
    //             if let Some(food) = game.foods.get(*id) {
    //                 // TODO: make food disappear
    //                 agent.consume_food(food);
    //             } else {
    //                 println!("food not found");
    //             }
    //         });
    //     }
    // });

    foods_to_remove.iter().for_each(|id| {
        game.foods.remove(id);

        // TODO: despawn food
    });
}

use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;

pub fn open_file_dialog(save_name: &str, folder: &str, extension: &str) -> Option<PathBuf> {
    let mut k: usize = 0;

    let mut default_path = std::env::current_dir().unwrap();
    default_path.push("saved");
    default_path.push(folder.to_string());
    let mut default_name: String;

    loop {
        default_name = save_name.to_string();
        default_name.push_str(&(k.to_string()));
        default_name.push_str(extension);

        default_path.push(&default_name);

        if !default_path.exists() {
            break;
        }
        default_path.pop();

        k += 1;
    }

    let res = rfd::FileDialog::new()
        .set_file_name(&default_name)
        .set_directory(&default_path)
        .save_file();
    println!("The user choose: {:#?}", &res);

    return res;
}

pub fn load_character_auto(
    keyboard: Res<Input<KeyCode>>,
    mut query: Query<&mut MarkerInstanceMatData>,
) {
    if keyboard.pressed(KeyCode::LControl) && keyboard.just_pressed(KeyCode::L) {
        let mut path = std::env::current_dir().unwrap();
        // default_path.push("saved");
        // default_path.push("groups");

        // cancel loading if user cancelled the file dialog
        // let mut path: std::path::PathBuf;
        // if let Some(chosen_path) = res.clone() {
        //     let path_some = chosen_path.get(0);
        //     if let Some(path_local) = path_some {
        //         path = path_local.clone();
        //     } else {
        //         return ();
        //     }
        // } else {
        //     return ();
        // }
        path = path.join("bah.cha");

        let mut file = std::fs::File::open(path).unwrap();

        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        println!("loaded: {}", contents);

        let loaded_character: CharacterSaveFormat = serde_json::from_str(&contents).unwrap();

        for mut character in query.iter_mut() {
            let character2 = character.as_mut();
            *character2 = loaded_character.clone().into();
        }
    }
}

pub fn load_character(keyboard: Res<Input<KeyCode>>, mut query: Query<&mut MarkerInstanceMatData>) {
    if keyboard.pressed(KeyCode::LControl) && keyboard.just_pressed(KeyCode::L) {
        let default_path = std::env::current_dir().unwrap();
        // default_path.push("saved");
        // default_path.push("groups");

        let res = rfd::FileDialog::new()
            .add_filter("text", &["cha"])
            .set_directory(&default_path)
            .pick_files();

        // cancel loading if user cancelled the file dialog
        let path: std::path::PathBuf;
        if let Some(chosen_path) = res.clone() {
            let path_some = chosen_path.get(0);
            if let Some(path_local) = path_some {
                path = path_local.clone();
            } else {
                return ();
            }
        } else {
            return ();
        }

        let mut file = std::fs::File::open(path).unwrap();

        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        println!("loaded: {}", contents);

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
    pub agent_id2: u32,
    pub atom_entity2: Entity,
    // pub acceleration2: Vec2,
    pub velocity2: Vec2,
    pub other_collision_mass2: f32,
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

        let closest_agents = kdtrees
            .agent_kdtree
            .nearest(&[agent.position.x, agent.position.y], 2, &squared_euclidean)
            .unwrap();

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

            if !agent_pair_checked.contains(&(closest_agent.id.kdtree_hash, agent.id.kdtree_hash)) {
                agent_pair_checked.push((closest_agent.id.kdtree_hash, agent.id.kdtree_hash));
                agent_pair_checked.push((agent.id.kdtree_hash, closest_agent.id.kdtree_hash));

                let body = &agent.body;
                let other_body = agent_bodies.get(&closest_agent.id.kdtree_hash).unwrap();

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

                        // let global_atom_pos = atom.atom_pos + agent.position;
                        // let other_global_atom_pos = other_atom.atom_pos + closest_agent.position;

                        let global_atom_pos = atom_transform1.translation.truncate();
                        let other_global_atom_pos = atom_transform2.translation.truncate();

                        // let atom1_size = ATOM_MULT * agent.mass * MASS_MULT;
                        // let atom2_size = ATOM_MULT * closest_agent.mass * MASS_MULT;

                        let dist = (global_atom_pos - other_global_atom_pos).length();

                        // let atom_test_dist = 0.05 * agent.mass * MASS_MULT * 0.5
                        //     + 0.05 * closest_agent.mass * MASS_MULT * 0.5;

                        // there is a collision. Must apply acceleration to atom
                        // if dist < (atom.atom_size + other_atom.atom_size) / 2.0 {
                        // if dist < (atom1_size + atom2_size) / 2.0 {
                        if dist < (agent.radius + closest_agent.radius) {
                            // let u1 = agent.position - agent.last_position;
                            // let u2 = closest_agent.position - closest_agent.last_position;

                            collisioned_agents.push(&closest_agent.id.kdtree_hash);
                            collisioned_agents.push(&agent.id.kdtree_hash);

                            let u1 = agent.velocity;
                            let u2 = Vec2::new(
                                closest_agent.look_at_angle.cos() * closest_agent.speed,
                                closest_agent.look_at_angle.sin() * closest_agent.speed,
                            );

                            // let u1 =

                            let m_ratio1 =
                                2.0 * closest_agent.mass / (closest_agent.mass + agent.mass);
                            let m_ratio2 = 2.0 * agent.mass / (closest_agent.mass + agent.mass);

                            let deltax1 = agent.position - closest_agent.position;
                            let deltax2 = closest_agent.position - agent.position;

                            let dot1 = (u1 - u2).dot(deltax1);
                            let dot2 = (u2 - u1).dot(deltax2);

                            let len_deltax1 = deltax1.length();
                            let len_deltax2 = deltax2.length();

                            let v1 = u1 - m_ratio1 * dot1 * deltax1 / (len_deltax1 * len_deltax1);
                            let v2 = u2 - m_ratio2 * dot2 * deltax2 / (len_deltax2 * len_deltax2);

                            let impulse1 = agent.mass * (v2 - v1);
                            let impulse2 = closest_agent.mass * (v1 - v2);

                            // impule = m * a * dt
                            let acceleration1 = impulse1 / agent.mass * 60.0; // 60 fps
                            let acceleration2 = impulse2 / closest_agent.mass * 60.0; // 60 fps

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
                                agent_id1: agent.id.kdtree_hash,
                                atom_entity1: *child1,
                                other_collision_mass1,
                                // velocity1: v1,
                                velocity1,
                                agent_id2: closest_agent.id.kdtree_hash,
                                atom_entity2: child2.clone(),
                                other_collision_mass2,
                                velocity2,
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

    let mut id_cache = Vec::new();
    for collision in collisions {
        let mut agent = game.agents.get_mut(&collision.agent_id1).unwrap();

        println!("collision: {:?}", id_cache);
        println!("kdtree_hash: {:?}", agent.id.kdtree_hash);
        if id_cache.contains(&agent.id.kdtree_hash) {
            panic!("nono");
        }
        id_cache.push(agent.id.kdtree_hash);
        // agent.body[collision.atom_index1].acceleration = collision.acceleration1;
        agent.last_position = agent.position - collision.velocity1;
        agent.just_collided = true;
        agent.other_collider_mass = collision.other_collision_mass1;
        let previous_mass = agent.mass;
        agent.previous_mass = agent.mass;

        let mut mass_exchange = 0.0;

        if agent.mass > agent.other_collider_mass {
            mass_exchange = agent.other_collider_mass * MASS_EXCHANGE_RATE;
        } else {
            mass_exchange = -agent.mass * MASS_EXCHANGE_RATE;
        }

        agent.mass = agent.mass + mass_exchange;

        collision_event.send(CollisionEvent {
            agent_id: agent.id.kdtree_hash,
            previous_mass,
        });

        let mut closest_agent = game.agents.get_mut(&collision.agent_id2).unwrap();

        println!("collision: {:?}", id_cache);
        if id_cache.contains(&closest_agent.id.kdtree_hash) {
            panic!("nono");
        }
        id_cache.push(closest_agent.id.kdtree_hash);

        closest_agent.last_position = closest_agent.position - collision.velocity2;
        // closest_agent.body[collision.atom_index2].acceleration = collision.acceleration2;
        closest_agent.just_collided = true;
        closest_agent.other_collider_mass = collision.other_collision_mass2;
        let previous_mass = closest_agent.mass;
        closest_agent.previous_mass = closest_agent.mass;

        // if closest_agent.mass > closest_agent.other_collider_mass {
        //     closest_agent.mass += closest_agent.other_collider_mass * MASS_EXCHANGE_RATE;
        // } else {
        //     closest_agent.mass -= closest_agent.mass * MASS_EXCHANGE_RATE;
        // }

        closest_agent.mass = closest_agent.mass - mass_exchange;

        collision_event.send(CollisionEvent {
            agent_id: closest_agent.id.kdtree_hash,
            previous_mass,
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
        if rng.gen::<f32>() < 0.1 {
            // if the past goal has been going on for too long, change it
            if time.seconds_since_startup() as f32 - agent.goal_time > agent.memory_time {
                for (seen_agent_id, agent_sighting) in agent.sensors.agent_sight.iter() {
                    //
                    let mass_ratio = agent.mass / (agent.mass + agent_sighting.mass);

                    if mass_ratio > 0.5 && mass_ratio < 0.8 {
                        if rng.gen::<f32>() < 0.2 {
                            agent.goal = Goal::Bully(AgentId {
                                kdtree_hash: seen_agent_id.clone(),
                            });
                            agent.goal_time = time.seconds_since_startup() as f32;
                            break;
                        }
                    }
                }
            }

            // if let Goal::None = agent.goal {
            //     agent.find_new_goal(time);
            // }
        }
    }
}

pub fn update_agent_properties(
    mut game: ResMut<Game>,
    mut agents_transform_query: Query<&mut Transform, With<AgentId>>,
    atoms_query: Query<&Children, With<AgentId>>,
    mut atom_transform_query: Query<&mut Transform, (With<Atom>, Without<AgentId>)>,
    mut collision_event: EventReader<CollisionEvent>,
) {
    for collision_info in collision_event.iter() {
        let mut agent = game.agents.get_mut(&collision_info.agent_id).unwrap();
        // for (id, agent) in game.agents.iter_mut() {
        let temp_mass = agent.mass;
        agent.radius = agent.mass * MASS_MULT * 0.5;
        agent.sensors.sight_range = agent.radius * 10.0;
        let mass_variation = agent.mass - collision_info.previous_mass;
        // println!("reducing mass: {}", mass_variation);

        let mut agent_quad_transform = agents_transform_query
            .get_mut(agent.entity.unwrap())
            .unwrap();

        agent_quad_transform.scale.x *= agent.mass / agent.previous_mass;
        agent_quad_transform.scale.y *= agent.mass / agent.previous_mass;

        let atoms = atoms_query.get(agent.entity.unwrap()).unwrap();
        for atom_entity in atoms.iter() {
            let mut atom_transform = atom_transform_query.get_mut(*atom_entity).unwrap();

            atom_transform.translation.x *= 1.0 + mass_variation / agent.mass;
            atom_transform.translation.y *= 1.0 + mass_variation / agent.mass;
            // atom_transform.scale.x *= 1.0 + mass_variation / agent.mass;
            // atom_transform.scale.y *= 1.0 + mass_variation / agent.mass;

            // atom_transform.translation.y = 0.0;

            atom_transform.scale.x *= 1.0 + mass_variation / agent.mass;
            atom_transform.scale.y *= 1.0 + mass_variation / agent.mass;
        }
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
