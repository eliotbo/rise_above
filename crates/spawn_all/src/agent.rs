use crate::util::*;
// use crate::vec2::*;

use bevy::prelude::*;

use rand::prelude::*;
use std::collections::HashMap;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Clone, Debug)]
pub struct Body {
    pub atom_pos: Vec2,
    pub atom_size: f32,
    pub acceleration: Vec2,
    // pub item_anchors: Vec2,
}

#[derive(Clone, Debug)]
pub struct Agent {
    pub id: AgentId,

    pub position: Vec2,
    pub last_position: Vec2,
    pub speed: f32,
    pub look_at_angle: f32,
    pub target_position: Vec2,

    pub body: Vec<Body>,
    pub just_collided: bool, // compute the atom damped bounce animation

    pub turning: Turning,
    pub acc: Acceleration,

    pub race: Race,
    pub social: Social,

    pub hp: f32,
    pub energy: f32,
    pub mass: f32,
    pub satiety: f32,
    pub power_usage: f32,
    pub memory_time: f32,

    pub goal: Goal,
    pub goal_time: f32,
    pub goal_status: AgentGoalStatus,
    pub goal_history: Vec<Goal>,

    pub sensors: Sensors,

    pub maybe_team_id: Option<u32>,
}

impl Agent {
    pub fn gen_random(stage: &GameStage, id: u32) -> Self {
        let mut rng = thread_rng();

        let position: Vec2;
        let mass: f32;
        let race: Race;
        let hp: f32;
        // let social_attributes: SocialAttributes;
        let hearing_range: f32;
        let sight_range: f32;
        let memory_time: f32;

        match stage {
            bottom @ GameStage::Bottom => {
                position = Vec2::new(
                    rng.gen_range(BOTTOM_LIMIT_X_MIN..BOTTOM_LIMIT_X_MAX),
                    rng.gen_range(0.0..BOTTOM_STAGE_LIMIT),
                );
                mass = rng.gen_range(0.0..BOTTOM_STAGE_LIMIT);
                hp = rng.gen_range(0.0..BOTTOM_STAGE_LIMIT);
                hearing_range = rng.gen_range(0.0..BOTTOM_STAGE_LIMIT);
                sight_range = rng.gen_range(0.0..BOTTOM_STAGE_LIMIT);
                race = Race::random_race(&bottom);
            }
            mid @ GameStage::Mid => {
                position = Vec2::new(
                    rng.gen_range(0.0..1.0),
                    rng.gen_range(BOTTOM_STAGE_LIMIT..MID_STAGE_LIMIT),
                );
                mass = rng.gen_range(BOTTOM_STAGE_LIMIT..MID_STAGE_LIMIT);
                hp = rng.gen_range(BOTTOM_STAGE_LIMIT..MID_STAGE_LIMIT);
                hearing_range = rng.gen_range(BOTTOM_STAGE_LIMIT..MID_STAGE_LIMIT);
                sight_range = rng.gen_range(BOTTOM_STAGE_LIMIT..MID_STAGE_LIMIT);
                race = Race::random_race(&mid);
                // social_attributes = race.gen_socials();
            }
            top @ GameStage::Top => {
                position = Vec2::new(
                    rng.gen_range(0.0..TOP_STAGE_LIMIT),
                    rng.gen_range(MID_STAGE_LIMIT..TOP_STAGE_LIMIT),
                );
                mass = rng.gen_range(MID_STAGE_LIMIT..TOP_STAGE_LIMIT);
                hp = rng.gen_range(MID_STAGE_LIMIT..TOP_STAGE_LIMIT);
                hearing_range = rng.gen_range(MID_STAGE_LIMIT..TOP_STAGE_LIMIT);
                sight_range = rng.gen_range(MID_STAGE_LIMIT..TOP_STAGE_LIMIT);
                race = Race::random_race(&top);
                // social_attributes = race.gen_socials();
            }
        };

        let race_attributes = race.gen_attributes();
        let social_attributes = race_attributes.social_attributes;
        memory_time = race_attributes.memory_time;

        let social = Social {
            social_attributes,
            ..Default::default()
        };

        let sensors = Sensors {
            hearing_range,
            sight_range,
            ..Default::default()
        };

        Agent {
            id: AgentId { kdtree_hash: id },
            position,
            mass,
            hp,
            social,
            sensors,
            memory_time,
            ..Default::default()
        }
    }

    // momentum from an other agent towards self. If an agent is charging,
    // this momentum will be high (depending on the mass and speed of the other agent)
    pub fn compute_agent_charging_momentum(&self, other_agent: &Agent) -> f32 {
        let towards_self_dir = self.position - other_agent.position;

        let charging_momentum = Vec2::new(
            other_agent.look_at_angle.cos() * other_agent.speed * other_agent.mass,
            other_agent.look_at_angle.sin() * other_agent.speed * other_agent.mass,
        )
        .dot(towards_self_dir);

        charging_momentum
    }

    pub fn compute_self_velocity(&self) -> Vec2 {
        Vec2::new(
            self.look_at_angle.cos() * self.speed,
            self.look_at_angle.sin() * self.speed,
        )
    }

    // pub fn compute_agentsight_charging_momentum(
    //     &self,
    //     other_agent: &AgentSight,
    //     atom1: Vec2,
    //     atom2: Vec2, // other
    // ) -> Vec2 {
    //     let towards_self_dir = atom1 - atom2;

    //     let charging_momentum = Vec2::new(
    //         other_agent.look_at_angle.cos() * other_agent.speed * other_agent.mass,
    //         other_agent.look_at_angle.sin() * other_agent.speed * other_agent.mass,
    //     )
    //     .dot(towards_self_dir);

    //     charging_momentum
    // }

    pub fn update_agent_sight(&mut self, time: f32, distance: f32, other_agent: &Agent) {
        //
        let charging_momentum = self.compute_agent_charging_momentum(other_agent);

        let new_agent_sight = AgentSight {
            time_of_last_sight: time,
            distance: distance,
            // thing: Sight::Agent(OtherAgentSight {
            id: other_agent.id.kdtree_hash,
            position: other_agent.position,
            last_position: other_agent.last_position,
            speed_along_itself: charging_momentum,
            feeling: other_agent.social.feeling,
            mass: other_agent.mass,
            speed: other_agent.speed,
            look_at_angle: other_agent.look_at_angle,
            // status: Status::Alive,
            // }),
        };

        self.sensors
            .agent_sight
            .insert(other_agent.id.clone().kdtree_hash, new_agent_sight);
    }

    pub fn update_item_sight(&mut self, time: f32, distance: f32, item: &Item) {
        // self.sensors.sight = data;
        if let Some(item_sight) = self.sensors.item_sight.get_mut(&item.id) {
            item_sight.time_of_last_sight = time;
            item_sight.distance = distance;
        } else {
            let new_item_sight = ItemSight {
                time_of_last_sight: time,
                distance: distance,
                id: item.id,
                item_type: item.item_type,
                position: item.position,
                mass: item.mass,
                range: item.range,
                damage: item.damage,
                hp: item.hp,
                // thing: Sight::Item(Item {
                //     id: item.id,
                //     item_type: item.item_type,
                //     position: item.position,
                //     mass: item.mass,
                //     range: item.range,
                //     damage: item.damage,
                //     hp: item.hp,
                // }),
            };

            self.sensors.item_sight.insert(item.id, new_item_sight);
        }
    }

    pub fn update_food_sight(&mut self, time: f32, distance: f32, food: &Food) {
        // self.sensors.sight = data;
        if let Some(food_sight) = self.sensors.food_sight.get_mut(&food.id) {
            food_sight.time_of_last_sight = time;
            food_sight.distance = distance;
        } else {
            let new_food_sight = FoodSight {
                time_of_last_sight: time,
                distance: distance,
                id: food.id,
                position: food.position,
                mass: food.mass,
                energy: food.energy,
                // thing: Sight::Food(Food {

                // }),
            };

            self.sensors.food_sight.insert(food.id, new_food_sight);
        }
    }

    pub fn forget_agents(&mut self, time: f32) {
        let mut to_remove: Vec<u32> = Vec::new();
        for (id, sight_data) in self.sensors.agent_sight.iter_mut() {
            if time - sight_data.time_of_last_sight > self.memory_time * 2.0 {
                to_remove.push(*id);
            }
        }

        for id in to_remove {
            self.sensors.agent_sight.remove(&id);
        }
    }

    pub fn forget_items(&mut self, time: f32) {
        let mut to_remove: Vec<u32> = Vec::new();
        for (id, sight_data) in self.sensors.item_sight.iter_mut() {
            if time - sight_data.time_of_last_sight > self.memory_time {
                to_remove.push(*id);
            }
        }

        for id in to_remove {
            self.sensors.item_sight.remove(&id);
        }
    }

    pub fn forget_food(&mut self, time: f32) {
        let mut to_remove: Vec<u32> = Vec::new();
        for (id, sight_data) in self.sensors.food_sight.iter_mut() {
            if time - sight_data.time_of_last_sight > self.memory_time / 2.0 {
                to_remove.push(*id);
            }
        }

        for id in to_remove {
            self.sensors.food_sight.remove(&id);
        }
    }

    pub fn consume_food(&mut self, food: &Food) {
        //
        self.mass += food.mass;
        self.energy += food.energy;
    }

    pub fn find_new_goal(&mut self, time: f32) {
        // restart goal timer
        self.goal_time = time;

        let mut rng = rand::thread_rng();

        let altruism = self.social.social_attributes.altruism;
        let aggro = self.social.social_attributes.aggressivity;
        let collect = self.social.social_attributes.collectioneur;

        let prob_aggro = aggro * 0.25;
        let prob_altruism = altruism * 0.3;
        let prob_collect = collect * 0.5;

        let items_in_sight = self.sensors.item_sight.clone();
        let agents_in_sight = self.sensors.agent_sight.clone();
        let food_in_sight = self.sensors.food_sight.clone();

        // if an item is in sight and the rng okays iit, then change the goal to get that item
        if !self.sensors.item_sight.is_empty() && rng.gen::<f32>() < prob_collect {
            //
            let mut closest_item = 0u32;
            let mut closest_dist = 1000000.0;
            for (id, items_in_sight) in items_in_sight.clone() {
                if items_in_sight.distance < closest_dist {
                    closest_item = id;
                    closest_dist = items_in_sight.distance;
                }
            }
            self.goal = Goal::Item(items_in_sight.get(&closest_item).unwrap().clone());

        // if an agent is in sight and the rng okays iit, then change the goal to go to that agent
        } else if !self.sensors.agent_sight.is_empty() && rng.gen::<f32>() < prob_altruism {
            //
            let a = agents_in_sight.iter().next().unwrap().1;
            self.goal = Goal::GoToAgent(AgentId { kdtree_hash: a.id });

        // if there is food in sight, go to closest
        } else if !self.sensors.food_sight.is_empty() && rng.gen::<f32>() < 0.95 {
            //
            //
            let a = food_in_sight.iter().next().unwrap().1;
            self.goal = Goal::Food(a.clone());
        }

        if rng.gen::<f32>() < prob_aggro {
            self.goal = Goal::SearchForAFight;
            return;
        } else if rng.gen::<f32>() < prob_altruism {
            self.goal = Goal::SearchTeam;
            return;
        } else if rng.gen::<f32>() < 0.5 {
            self.goal = Goal::SearchForFood;
            return;
        } else {
            self.goal = Goal::None;
        }
    }

    pub fn act(&mut self, agent_positions: &HashMap<u32, Vec2>) {
        //
        let mut rng = thread_rng();

        match self.goal.clone() {
            //
            Goal::GoToAgent(agent_id) => {
                self.target_position = *agent_positions.get(&agent_id.kdtree_hash).unwrap();
            }
            Goal::Food(food_sight) => {
                self.target_position = food_sight.position;
            }
            Goal::Item(item_sight) => {
                self.target_position = item_sight.position;
            }
            Goal::Bully(agent_id) => {
                self.target_position = *agent_positions.get(&agent_id.kdtree_hash).unwrap();
            }
            Goal::Flee(from) => {
                let fleeing_direction = (self.position - from).normalize();
                self.target_position = self.position + fleeing_direction * self.mass * 100.0;
            }
            // meander around in search for whatever the goal is
            Goal::SearchForAFight
            | Goal::SearchTeam
            | Goal::SearchForFood
            | Goal::SearchForItem => {
                self.target_position = self.position
                    + Vec2::new(
                        self.look_at_angle.cos() + (rng.gen::<f32>() - 0.5) * 0.2,
                        self.look_at_angle.sin() + (rng.gen::<f32>() - 0.5) * 0.2,
                    );
            }
            _ => {}
        }

        let theta = self.target_position.y.atan2(self.target_position.x);
        let delta_theta = theta - self.look_at_angle; // if positive, turn left
        if delta_theta > 0.01 {
            self.turning = Turning::Left;
        } else if delta_theta < 0.01 {
            self.turning = Turning::Right;
        } else {
            self.turning = Turning::None;
        }

        if let Goal::None = self.goal {
            self.turning = Turning::None;
            self.acc = Acceleration::None;
        } else {
            self.acc = Acceleration::None;
        }
    }

    pub fn react_to_collision(
        &mut self,
        attacker: &AgentId,
        attacker_mass: f32,
        attacker_position: Vec2,
        time: f32,
    ) {
        //
        let offset = 0.0;
        let ratio = attacker_mass / self.mass;
        let p_of_fleeing = sigmoid(ratio, -1.0, 0.98, 0.02, 10.0, offset);
        let mut rng = thread_rng();
        if rng.gen::<f32>() < p_of_fleeing {
            self.goal = Goal::Flee(attacker_position);
        } else {
            self.goal = Goal::Bully(attacker.clone());
        }
        self.goal_time = time;
    }

    pub fn react_to_threat() {
        unimplemented!()
    }

    pub fn decision_making(&mut self, time: f32) {
        // if attacked
        // 1. attack back
        // 2. run away
        // 3. do nothing
        // 4. ask to team
    }

    // // invitation to already existing team
    // pub fn process_invitation_to_team(&mut self, team: &Team) -> bool {
    //     //
    //     let r_tot = team.total_mass / self.mass - 1.0;

    //     let r_max = (team.maximum_mass / self.mass).clamp(0.3, 1.0);
    //     let up = 0.3 * r_max;

    //     let slope = 10.0;
    //     let attr = self.social.social_attributes.altruism / 5.0;

    //     let p_of_accept = sigmoid(r_tot, -1.0, up, 0.01, slope, attr);

    //     let mut rng = thread_rng();
    //     let rng_val = rng.gen::<f32>();
    //     if rng_val < p_of_accept {
    //         self.maybe_team_id = Some(team.id);

    //         // TODO: set goal to follow team leader

    //         return true;
    //     }
    //     return false;
    // }

    // // decisions needs to last some time between 5 and 25 seconds.
    // pub fn process_invitation_to_new_team(
    //     &mut self,
    //     other_agent: &Agent,
    //     maybe_team_id: Option<u32>,
    //     agent_or_team_mass: f32,
    // ) -> bool {
    //     //
    //     false
    // }
}

impl Default for Agent {
    fn default() -> Self {
        let mut rng = thread_rng();

        let id: u32 = rng.gen();
        let eye_angle: f32 = rng.gen();

        let sensors = Sensors::default();

        return Self {
            // id: id,
            // position: Vec2::ZERO,
            // requested_position: Vec2::ZERO,
            //
            id: AgentId { kdtree_hash: id },
            position: Vec2::new(0.0, 0.0),
            last_position: Vec2::new(0.0, 0.0),
            target_position: Vec2::new(0.0, 0.0),
            speed: 0.0,
            look_at_angle: eye_angle * 3.1415 * 2.0,

            body: vec![],
            just_collided: false,

            turning: Turning::None,
            acc: Acceleration::Forward,

            race: Race::random_race(&GameStage::Bottom),
            social: Social::default(),

            hp: 1.0,
            mass: 0.1,
            energy: 1.0,
            satiety: 1.0,
            memory_time: 4.0,

            power_usage: 0.0,

            goal_status: AgentGoalStatus::None,
            goal_time: 0.0,
            goal: Goal::None,
            goal_history: vec![],

            sensors: sensors,

            maybe_team_id: None,
        };
    }
}

// impl Social {
//     pub fn gen_socials(&mut self, race: Race) {
//         self.social_attributes = race.gen_socials();
//     }
// }

#[derive(Clone, Debug)]
pub struct Sensors {
    pub hearing_range: f32,
    pub sight_range: f32,

    pub agent_sight: HashMap<u32, AgentSight>,
    pub food_sight: HashMap<u32, FoodSight>,
    pub item_sight: HashMap<u32, ItemSight>,
    pub hearing: HashMap<u32, HearingData>,
}

impl Default for Sensors {
    fn default() -> Self {
        Self {
            hearing_range: 1.0,
            sight_range: 1.0,
            agent_sight: HashMap::new(),
            item_sight: HashMap::new(),
            food_sight: HashMap::new(),
            hearing: HashMap::new(),
        }
    }
}

// pub enum Status {
//     Alive,
//     Dead,
//     Unknown,
// }

#[derive(Clone, Debug)]
pub enum Turning {
    Left,
    Right,
    None,
}

#[derive(Clone, Debug)]
pub enum Acceleration {
    Forward,
    Backward,
    None,
}

#[derive(Clone, Debug)]
pub struct AgentSight {
    pub time_of_last_sight: f32,
    pub distance: f32,
    pub id: u32,
    pub position: Vec2,
    pub last_position: Vec2,
    pub speed_along_itself: f32,
    pub feeling: Feeling,
    pub mass: f32,
    pub speed: f32,
    pub look_at_angle: f32,
    // pub status: Status,
}

#[derive(Clone, Debug)]
pub struct ItemSight {
    pub time_of_last_sight: f32,
    pub distance: f32,
    // pub thing: Sight,
    pub id: u32,
    pub item_type: ItemType,
    pub position: Vec2,
    pub mass: f32,
    pub range: f32,
    pub damage: f32,
    pub hp: f32,
}

#[derive(Clone, Debug)]
pub struct FoodSight {
    pub time_of_last_sight: f32,
    pub distance: f32,
    // pub thing: Sight,
    pub position: Vec2,
    pub energy: f32,
    pub mass: f32,
    pub id: u32,
}

// TODO: Not priority but, ItemSight and FoodSight maybe should be separate types
#[derive(Clone, Debug)]
pub struct OtherAgentSight {
    pub id: u32,
    pub position: Vec2,
    pub speed_along_itself: f32,
    pub feeling: Feeling,
}

#[derive(Clone, Debug)]
pub enum Sight {
    Agent(OtherAgentSight),
    Food(Food),
    Item(Item),
    None,
}

#[derive(Clone, Debug)]
pub struct HearingData {
    pub time_of_hearing: f32,
    pub distance: f32,
    pub things: Vec<Hearing>,
}

#[derive(Clone, Debug)]
pub enum Hearing {
    Agent(Direction),
    Weapon(Direction),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AgentGoalStatus {
    None,
    LookingForGoal,
    WorkingOnIt,
    Completed,
    Error,
}

#[derive(Debug, Clone)]
pub enum Goal {
    None,
    SearchForAFight,
    SearchTeam,
    SearchForFood,
    SearchForItem,
    FindPartner(f32), // mass
    GoToAgent(AgentId),
    Food(FoodSight),
    Flee(Vec2), // direction
    Item(ItemSight),
    Bully(AgentId),
}

#[derive(Debug, EnumIter, Copy, Clone)]
pub enum Feeling {
    Neutral,
    Happy,
    Sadness,
    Angry,
}

impl Feeling {
    pub fn random_feeling() -> Feeling {
        let mut rng = rand::thread_rng();
        return Feeling::iter().choose(&mut rng).unwrap();
    }
}

#[derive(Debug, EnumIter, Clone)]
pub enum RaceBottom {
    Ameoba,
    StratolopusArealus,
}

#[derive(Debug, EnumIter, Clone)]
pub enum RaceMid {
    Piko,
    Seahorse,
}

#[derive(Debug, EnumIter, Clone)]
pub enum RaceTop {
    Squid,
    Whale,
}

#[derive(Debug, Clone)]
pub enum Race {
    Bottom(RaceBottom),
    Mid(RaceMid),
    Top(RaceTop),
}

pub struct RaceAttributes {
    pub social_attributes: SocialAttributes,
    pub memory_time: f32,
}

impl Race {
    pub fn random_race(stage: &GameStage) -> Race {
        let mut rng = rand::thread_rng();

        let race = match stage {
            GameStage::Bottom => Race::Bottom(RaceBottom::iter().choose(&mut rng).unwrap()),
            GameStage::Mid => Race::Mid(RaceMid::iter().choose(&mut rng).unwrap()),
            GameStage::Top => Race::Top(RaceTop::iter().choose(&mut rng).unwrap()),
        };

        return race;
    }

    // pub fn gen_memory_time(&self)

    pub fn gen_attributes(&self) -> RaceAttributes {
        let mut rng = thread_rng();

        let socials = match self {
            // Bottom
            Race::Bottom(RaceBottom::Ameoba) => RaceAttributes {
                social_attributes: SocialAttributes {
                    aggressivity: rng.gen_range(0.0..0.4),
                    altruism: rng.gen_range(0.0..0.4),
                    collectioneur: rng.gen_range(0.0..0.5),
                },
                memory_time: rng.gen_range(0.0..0.5),
            },

            Race::Bottom(RaceBottom::StratolopusArealus) => RaceAttributes {
                social_attributes: SocialAttributes {
                    aggressivity: rng.gen_range(0.2..0.3),
                    altruism: rng.gen_range(0.3..0.5),
                    collectioneur: rng.gen_range(0.0..0.5),
                },
                memory_time: rng.gen_range(0.5..1.5),
            },

            // Mid
            Race::Mid(RaceMid::Piko) => RaceAttributes {
                social_attributes: SocialAttributes {
                    aggressivity: rng.gen_range(0.5..0.7),
                    altruism: rng.gen_range(0.0..0.4),
                    collectioneur: rng.gen_range(0.0..0.5),
                },
                memory_time: rng.gen_range(2.0..4.0),
            },

            Race::Mid(RaceMid::Seahorse) => RaceAttributes {
                social_attributes: SocialAttributes {
                    aggressivity: rng.gen_range(0.0..0.2),
                    altruism: rng.gen_range(0.0..0.5),
                    collectioneur: rng.gen_range(0.0..0.5),
                },
                memory_time: rng.gen_range(3.0..3.5),
            },

            // Top
            Race::Top(RaceTop::Squid) => RaceAttributes {
                social_attributes: SocialAttributes {
                    aggressivity: rng.gen_range(0.1..0.3),
                    altruism: rng.gen_range(0.4..0.9),
                    collectioneur: rng.gen_range(0.0..0.5),
                },
                memory_time: rng.gen_range(3.0..5.0),
            },
            Race::Top(RaceTop::Whale) => RaceAttributes {
                social_attributes: SocialAttributes {
                    aggressivity: rng.gen_range(0.1..0.9),
                    altruism: rng.gen_range(0.6..0.9),
                    collectioneur: rng.gen_range(0.0..0.5),
                },
                memory_time: rng.gen_range(5.0..10.0),
            },
            // other?
            _ => RaceAttributes {
                social_attributes: SocialAttributes {
                    aggressivity: rng.gen_range(0.0..1.0),
                    altruism: rng.gen_range(0.0..1.0),
                    collectioneur: rng.gen_range(0.0..0.5),
                },
                memory_time: rng.gen_range(1.0..1.01),
            },
        };

        return socials;
    }
}

#[derive(Clone, Debug, Component)]
pub struct AgentId {
    pub kdtree_hash: u32,
    // pub maybe_AgentId: Option<AgentId>,
}

#[derive(Clone, Debug)]
pub struct SocialAttributes {
    pub aggressivity: f32,
    pub altruism: f32,
    pub collectioneur: f32,
}

#[derive(Clone, Debug)]
pub struct PartnerData {
    pub time_since_partnered: f32,
    pub feeling: Feeling,
}

#[derive(Clone, Debug)]
pub struct Social {
    pub agent_whom_asked: Option<AgentId>,
    pub asked_to_agent: Option<AgentId>,
    pub partners: HashMap<u32, PartnerData>, // the value is time since partnered

    pub feeling: Feeling,
    pub social_attributes: SocialAttributes,
}

impl Default for Social {
    fn default() -> Self {
        let mut rng = thread_rng();
        let aggressivity: f32 = rng.gen();
        let altruism: f32 = rng.gen();
        let collectioneur: f32 = rng.gen();

        return Social {
            partners: HashMap::new(),
            agent_whom_asked: None,
            asked_to_agent: None,

            feeling: Feeling::Neutral,
            social_attributes: SocialAttributes {
                aggressivity,
                altruism,
                collectioneur,
            },
        };
    }
}

pub fn agent_decisions(game: &mut Game) {
    let mut agent_goals = HashMap::new();

    game.agents.iter().for_each(|(id, agent)| {
        agent_goals.insert(id, agent.goal.clone());
    });

    let mut rng = rand::thread_rng();
    let time = game.time;

    for (id, agent) in game.agents.iter_mut() {
        // make a decision once per 10 frames on average
        if rng.gen::<f32>() < 0.1 {
            // if the past goal has been going on for too long, change it
            if time - agent.goal_time > agent.memory_time {
                agent.find_new_goal(time);
            }

            if let Goal::None = agent.goal {
                agent.find_new_goal(time);
            }
        }
    }
}

pub fn agent_action(game: &mut Game) {
    let mut rng = rand::thread_rng();

    let agent_positions = game
        .agents
        .iter()
        .map(|(id, x)| (*id, x.position))
        .collect::<HashMap<_, _>>();

    for (_id, agent) in game.agents.iter_mut() {
        if rng.gen::<f32>() < 0.5 {
            agent.act(&agent_positions);
        }
    }
}

// #[derive(Clone, Debug)]
// pub struct InviteToTeamEvent {
//     pub inviter: AgentId,
//     pub invitee: AgentId,

//     pub inviter_team: Team,
//     // pub invitee_team: Option<u32>,
// }

// pub fn receive_invite_to_team(game: &mut Game, invite_event: InviteToTeamEvent) {
//     // let inviter_id = invite_event.inviter.clone();
//     // if let Some(requester_agent) = game.agents.get(&invite_event.clone().inviter.kdtree_hash) {

//     let mut do_attack = false;
//     let mut do_flee = false;
//     let mut do_nothing = false;
//     let mut invitee_position = Vec2::new(0.0, 0.0);
//     if let Some(invited_agent) = game.agents.get_mut(&invite_event.invitee.kdtree_hash) {
//         //
//         let has_accepted = invited_agent.process_invitation_to_team(&invite_event.inviter_team);

//         if has_accepted {
//             if let Some(team) = game.teams.get_mut(&invite_event.inviter_team.id) {
//                 //
//                 team.agents.push(invite_event.invitee.clone());
//             }
//         } else {
//             // TODO: update the agent's social: change goal to flee
//             invitee_position = invited_agent.position;

//             let mut rng = rand::thread_rng();
//             let n: f32 = rng.gen();
//             if n < 0.2 {
//                 do_attack = true;
//             } else if n < 0.4 {
//                 do_flee = true;
//             } else {
//                 do_nothing = true;
//             }
//         }
//     }

//     // todo: better as a function with an event
//     // Response of the inviter to a negative answer fromt he invitee
//     if let Some(mut inviter_agent) = game.agents.get_mut(&invite_event.inviter.kdtree_hash) {
//         if do_flee {
//             let fleeing_direction = (inviter_agent.position - invitee_position).normalize();
//             inviter_agent.goal = Goal::Flee(fleeing_direction);
//         }

//         if do_attack {
//             inviter_agent.goal = Goal::Bully(invite_event.invitee);
//         }

//         if do_nothing {
//             inviter_agent.goal = Goal::None;
//         }
//     }
// }

//
// decision is based on (in priority order):
// 1. if the agent is being charged by a non-team member, either
//     a. attack
//     b. flee
//     c. do nothing
//     in any case, the agent's goals will be locked for 20 seconds (depends on memory)
//
// 2. if the agent is in a team, and a team member has been attacked,
//    a. attack the attacker
//    b. flee (if the attacker is too massive)
//
// 3. if the agent is in a team, and a team member is attacking an outsider,
//    a. attack the outsider
//    b. stay far (if the outsider is too massive)
//    c. attack the team member (if they are greedy)
//
// 4. if the agent is in a team, and a team member instigates a fight with another team member,
//    a. attack the most greedy
//    b. stay away
//    c. leave the party
//
// 5. if the agent is asked to join a team,
//    a. accept
//    b. refuse
//    c. attack the requester
//
// 6. if the agent receives a negative answer from a potential team member,
//    a. attack
//    b. flee
//    c. do nothing
//
// 6. if the agent receives a positive answer from a potential team member,
//
