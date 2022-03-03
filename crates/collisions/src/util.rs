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

// use crate::vec2::*;

// use bevy::prelude::*;

pub const NUM_AGENTS: usize = 200;
pub const NUM_ITEMS: usize = 10;
pub const NUM_FOODS: usize = 100;

// stage 1: y position from 0 to 0.01;
// stage 2: y position from 0.01 to 0.1;
// stage 3: y position from 0.1 to 1.0;

pub const BOTTOM_STAGE_LIMIT: f32 = 0.05;
pub const MID_STAGE_LIMIT: f32 = 0.2;
pub const TOP_STAGE_LIMIT: f32 = 1.0;

pub const BOTTOM_LIMIT_X_MIN: f32 = 0.45;
pub const BOTTOM_LIMIT_X_MAX: f32 = 0.55;

// start sight range is 1.0
// stage 2 is 10.0
// stage 3 is 100.0

type TeamId = u32;

#[derive(Clone, Debug)]
pub struct Team {
    pub id: TeamId,
    pub total_mass: f32,
    pub maximum_mass: f32,
    pub agents: Vec<AgentId>,
}

pub struct Game {
    pub time: f32,
    pub game_stage: GameStage,
    pub agents: HashMap<u32, Agent>,
    pub items: HashMap<u32, Item>,
    pub foods: HashMap<u32, Food>,
    pub agent_kdtree: KdTree<f32, u32, [f32; 2]>,
    pub item_kdtree: KdTree<f32, u32, [f32; 2]>,
    pub food_kdtree: KdTree<f32, u32, [f32; 2]>,

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
            agent_kdtree: Self::gen_agent_kdtree(&agents),
            item_kdtree: Self::gen_item_kdtree(&items),
            food_kdtree: Self::gen_food_kdtree(&foods),
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

        let id: u32 = rng.gen();

        (0..num_foods).for_each(|_| {
            // let stage: usize = rng.gen_range(0..3);
            let stage = GameStage::iter().choose(&mut rng).unwrap();
            match stage {
                GameStage::Bottom => {
                    let y: f32 = rng.gen_range(0.0..BOTTOM_STAGE_LIMIT);
                    let x: f32 = rng.gen_range(BOTTOM_LIMIT_X_MIN..BOTTOM_LIMIT_X_MAX);

                    foods.insert(
                        id,
                        Food {
                            position: Vec2::new(x, y),
                            energy: rng.gen_range(0.0..0.02),
                            mass: rng.gen_range(0.0..0.02),
                            id,
                            acc: Vec2::new(0.0, 0.0),
                        },
                    );
                }

                GameStage::Mid => {
                    let y: f32 = rng.gen_range(BOTTOM_STAGE_LIMIT..MID_STAGE_LIMIT);
                    let x: f32 = rng.gen_range(0.0..1.0);

                    foods.insert(
                        id,
                        Food {
                            position: Vec2::new(x, y),
                            energy: rng.gen_range(BOTTOM_STAGE_LIMIT..MID_STAGE_LIMIT),
                            mass: rng.gen_range(BOTTOM_STAGE_LIMIT..MID_STAGE_LIMIT),
                            id,
                            acc: Vec2::new(0.0, 0.0),
                        },
                    );
                }

                GameStage::Top => {
                    let y: f32 = rng.gen_range(MID_STAGE_LIMIT..TOP_STAGE_LIMIT);
                    let x: f32 = rng.gen_range(0.0..TOP_STAGE_LIMIT);

                    foods.insert(
                        id,
                        Food {
                            position: Vec2::new(x, y),
                            energy: rng.gen_range(MID_STAGE_LIMIT..TOP_STAGE_LIMIT),
                            mass: rng.gen_range(MID_STAGE_LIMIT..TOP_STAGE_LIMIT),
                            id,
                            acc: Vec2::new(0.0, 0.0),
                        },
                    );
                }
                _ => {}
            }
        });

        foods
    }

    pub fn gen_agent_kdtree(agents: &HashMap<u32, Agent>) -> KdTree<f32, u32, [f32; 2]> {
        let dimensions = 2;
        let mut kdtree = KdTree::with_capacity(dimensions, NUM_AGENTS);
        agents.iter().for_each(|(id, agent)| {
            kdtree
                .add([agent.position.x, agent.position.y], *id)
                .unwrap();
        });

        kdtree
    }

    pub fn update_agent_kdtree(&mut self) {
        let dimensions = 2;
        let mut kdtree = KdTree::with_capacity(dimensions, NUM_AGENTS);
        self.agents.iter().for_each(|(id, agent)| {
            kdtree
                .add([agent.position.x, agent.position.y], *id)
                .unwrap();
        });

        self.agent_kdtree = kdtree;
    }

    pub fn gen_item_kdtree(items: &HashMap<u32, Item>) -> KdTree<f32, u32, [f32; 2]> {
        let dimensions = 2;
        let mut kdtree = KdTree::with_capacity(dimensions, 9);
        items.iter().for_each(|(id, item)| {
            kdtree.add([item.position.x, item.position.y], *id).unwrap();
        });
        kdtree
    }

    pub fn gen_food_kdtree(foods: &HashMap<u32, Food>) -> KdTree<f32, u32, [f32; 2]> {
        let dimensions = 2;
        let mut kdtree = KdTree::with_capacity(dimensions, 9);
        foods.iter().for_each(|(id, food)| {
            kdtree.add([food.position.x, food.position.y], *id).unwrap();
        });
        kdtree
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

fn see(game: &mut Game) {
    let all_agents = game.agents.clone();
    let all_items = game.items.clone();
    let all_foods = game.foods.clone();

    for (hash_id, mut agent) in &mut game.agents {
        // checks if agent is close enough to be seen
        if let Ok(dist_id_array) = game.agent_kdtree.within(
            &[agent.position.x, agent.position.y],
            agent.sensors.sight_range,
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
                agent.update_agent_sight(game.time, dist, other_agent);

                // despawn and spawn green cube upon seeing
                // }
            }
        }

        if let Ok(dist_id_array) = game.item_kdtree.within(
            &[agent.position.x, agent.position.y],
            agent.sensors.sight_range,
            &squared_euclidean,
        ) {
            //
            // agent.sensors.sight.

            for (dist, id) in dist_id_array {
                let item = all_items.get(&id).unwrap();
                // let other_agent_pos = other_agent.position;
                let direction_to_item = item.position - agent.position;

                // let dir_to_other_angle = direction_to_other_agent.y.atan2(direction_to_other_agent.x);
                let agent_looking_direction =
                    Vec2::new(agent.look_at_angle.cos(), agent.look_at_angle.sin());
                // if the angles differ by more than 180 degrees, then the other agent is on the other side of the agent
                // if agent_looking_direction.dot(direction_to_item) > 0.0 {
                agent.update_item_sight(game.time, dist, item);

                // despawn and spawn green cube upon seeing
                // }
            }
        }

        if let Ok(dist_id_array) = game.food_kdtree.within(
            &[agent.position.x, agent.position.y],
            agent.sensors.sight_range,
            &squared_euclidean,
        ) {
            //
            // agent.sensors.sight.

            for (dist, id) in dist_id_array {
                let food = all_foods.get(&id).unwrap();
                // let other_agent_pos = other_agent.position;
                let direction_to_food = food.position - agent.position;

                // let dir_to_other_angle = direction_to_other_agent.y.atan2(direction_to_other_agent.x);
                let agent_looking_direction =
                    Vec2::new(agent.look_at_angle.cos(), agent.look_at_angle.sin());
                // if the angles differ by more than 180 degrees, then the other agent is on the other side of the agent
                // if agent_looking_direction.dot(direction_to_food) > 0.0 {
                agent.update_food_sight(game.time, dist, food);

                // despawn and spawn green cube upon seeing
                // }
            }
        }
    }
}

fn forget(game: &mut Game) {
    let mut rng = thread_rng();
    for (_hash_id, mut agent) in &mut game.agents {
        // run once every ten frames on average
        if rng.gen::<f32>() < 0.1 {
            agent.forget_agents(game.time);
            agent.forget_items(game.time);
            agent.forget_food(game.time);
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

pub fn compute_acc_food(game: &mut Game) {
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

    let food_kd_tree = &game.food_kdtree;

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

pub fn eat_food(game: &mut Game) {
    let mut foods_to_remove: Vec<u32> = Vec::new();

    game.agents.iter_mut().for_each(|(_, mut agent)| {
        //
        // TODO: tweak this
        let agent_food_radius = agent.mass * 0.5;
        if let Ok(dist_id_array) = game.food_kdtree.within(
            &[agent.position.x, agent.position.y],
            agent_food_radius,
            &squared_euclidean,
        ) {
            //
            dist_id_array.iter().for_each(|(_dist, id)| {
                foods_to_remove.push(**id);
                if let Some(food) = game.foods.get(*id) {
                    agent.consume_food(food);
                } else {
                    println!("food not found");
                }
            });
        }
    });

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

pub fn update_agent_kdtree(game: &mut Game) {
    game.update_agent_kdtree();
}

pub struct AgentCollisionInfo {
    pub id1: u32,
    pub atom_index1: usize,
    pub acceleration1: Vec2,
    pub id2: u32,
    pub atom_index2: usize,
    pub acceleration2: Vec2,
}
pub fn collisions(game: &mut Game) {
    let agent_bodies = game
        .agents
        .iter()
        .map(|(id, x)| (*id, x.body.clone()))
        .collect::<HashMap<_, _>>();

    let mut collisions: Vec<AgentCollisionInfo> = Vec::new();

    game.agents.iter().for_each(|(id, agent)| {
        // let agent_pos = agent_positions.get(id).unwrap();

        let mut agent_pair_checked = Vec::new();
        agent
            .sensors
            .agent_sight
            .iter()
            .for_each(|(id, other_agent)| {
                // AgentCollisionInfo {
                //     id: *id,
                //     distance: other_agent.distance,
                //     charging_momentum: other_agent.speed_along_itself,
                //     mass: other_agent.mass,
                // }

                // perhaps a collision
                if other_agent.distance < (other_agent.mass + agent.mass) * 1.00 {
                    // 1.00 = buffer distance?

                    if !agent_pair_checked.contains(&(other_agent.id, agent.id.kdtree_hash)) {
                        agent_pair_checked.push((other_agent.id, agent.id.kdtree_hash));
                        agent_pair_checked.push((agent.id.kdtree_hash, other_agent.id));

                        let body = &agent.body;
                        let other_body = agent_bodies.get(id).unwrap();

                        body.iter().enumerate().for_each(|(j, atom)| {
                            other_body.iter().enumerate().for_each(|(k, other_atom)| {
                                let dist = (atom.atom_pos - other_atom.atom_pos).length();
                                // there is a collision. Must apply acceleration to atom
                                if dist
                                    < agent.mass * atom.atom_size
                                        + other_agent.mass * other_atom.atom_size
                                {
                                    // let u1 = agent.position - agent.last_position;
                                    // let u2 = other_agent.position - other_agent.last_position;

                                    let u1 = agent.velocity;
                                    let u2 = Vec2::new(
                                        other_agent.look_at_angle.cos() * other_agent.speed,
                                        other_agent.look_at_angle.sin() * other_agent.speed,
                                    );

                                    // let u1 =

                                    let m_ratio1 =
                                        2.0 * other_agent.mass / (other_agent.mass + agent.mass);
                                    let m_ratio2 =
                                        2.0 * agent.mass / (other_agent.mass + agent.mass);

                                    let deltax1 = agent.position - other_agent.position;
                                    let deltax2 = other_agent.position - agent.position;

                                    let dot1 = (u1 - u2).dot(deltax1);
                                    let dot2 = (u2 - u1).dot(deltax2);

                                    let v1 =
                                        u1 - m_ratio1 * dot1 * deltax1 / deltax1.length().powf(2.0);
                                    let v2 =
                                        u2 - m_ratio2 * dot2 * deltax2 / deltax2.length().powf(2.0);

                                    let impulse1 = agent.mass * (v2 - v1);
                                    let impulse2 = other_agent.mass * (v1 - v2);

                                    // impule = m * a * dt
                                    let acceleration1 = impulse1 / agent.mass * 60.0; // 60 fps
                                    let acceleration2 = impulse2 / other_agent.mass * 60.0; // 60 fps

                                    let collision = AgentCollisionInfo {
                                        id1: agent.id.kdtree_hash,
                                        atom_index1: j,
                                        acceleration1,
                                        id2: other_agent.id,
                                        atom_index2: k,
                                        acceleration2,
                                    };
                                    collisions.push(collision);
                                }
                            });
                        });
                    }
                }
            });
    });
    for collision in collisions {
        let mut agent = game.agents.get_mut(&collision.id1).unwrap();
        agent.body[collision.atom_index1].acceleration = collision.acceleration1;
        agent.just_collided = true;

        let mut other_agent = game.agents.get_mut(&collision.id2).unwrap();
        other_agent.body[collision.atom_index2].acceleration = collision.acceleration2;
        other_agent.just_collided = true;
    }
}
