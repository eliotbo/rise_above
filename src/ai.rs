use itertools::izip;
use rand::prelude::*;
use strum::IntoEnumIterator; // 0.17.1
use strum_macros::EnumIter;

use bevy::prelude::*;

#[derive(Clone, Debug)]
pub struct Agent {
    pub entity: Option<Entity>,
    pub id: u32,
    pub position: Vec2,
    pub requested_position: Vec2,
    pub tribe: Tribe,
    pub hunger: f32,
    pub aggressivity: f32,
    pub altruism: f32,
    pub age: f32,
    pub hp: f32,
    pub emotion: Emotion,
    pub power_usage: f32,
    pub tiredness: f32,
    pub look_at_angle: f32,
    pub speed: f32,
    pub latest_goal: Goal,
    pub goal_status: AgentGoalStatus,
    pub partners: Vec<u32>,
    pub agents_whom_asked: Vec<Entity>,
    pub asked_to_agents: Vec<Entity>,
    pub sensors: Sensors,
}

impl<'a> Default for Agent {
    fn default() -> Self {
        let mut rng = thread_rng();
        let id: u32 = rng.gen();
        let eye_angle_0_1: f32 = rng.gen();
        let age_0_1: f32 = rng.gen();
        let speed_0_1: f32 = rng.gen();
        let tired_0_1: f32 = rng.gen();
        let power_usage_0_1: f32 = rng.gen();
        let aggressivity_0_1: f32 = rng.gen();
        let altruism_0_1: f32 = rng.gen();
        // let id: u32 = rng.gen();
        let sensors = Sensors {
            sight: Vec::new(),
            hearing: Vec::new(),
        };

        return Self {
            entity: None,
            id: id,
            position: Vec2::ZERO,
            requested_position: Vec2::ZERO,
            tribe: Tribe::random_tribe(),
            hunger: 0.0,
            aggressivity: aggressivity_0_1,
            altruism: altruism_0_1,
            age: age_0_1,
            hp: 100.0,
            emotion: Emotion::random_emotion(),
            power_usage: power_usage_0_1 * 200.0,
            tiredness: tired_0_1,
            look_at_angle: eye_angle_0_1,
            speed: (speed_0_1 / 2.0 + 0.5) * 1.0,
            goal_status: AgentGoalStatus::None,
            // id: id,
            partners: Vec::new(),
            agents_whom_asked: Vec::new(),
            asked_to_agents: Vec::new(),
            latest_goal: Goal::None,
            sensors: sensors,
        };
    }
}

pub type Position = Vec2;

#[derive(Clone, Debug)]
pub struct Sensors {
    pub sight: Vec<Sight>,
    pub hearing: Vec<Hearing>,
}
#[derive(Clone, Debug)]
pub enum Sight {
    Agent(u32, Position, Emotion),
    Food(Position),
    Ore,
}

#[derive(Clone, Debug)]
pub enum Ore {
    Iron(Position),
    Copper(Position),
    Coal(Position),
    Petrol(Position),
}

#[derive(Clone, Debug)]
pub enum Food {
    Mango,
    Coconut,
}

#[derive(Clone, Debug)]
pub enum Hearing {
    Agent(Direction),
    Weapon(Direction),
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

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AgentGoalStatus {
    None,
    LookingForGoal,
    WorkingOnIt,
    Completed,
    Error,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Goal {
    None,
    FindPartner,
}

#[derive(Debug, EnumIter, Clone)]
pub enum Emotion {
    Neutral,
    Sadness,
    Disgust,
    Surprise,
    Bored,
    Anxious,
    Sleepy,
    Angry,
}

impl Emotion {
    pub fn random_emotion() -> Emotion {
        let mut rng = rand::thread_rng();
        return Emotion::iter().choose(&mut rng).unwrap();
    }
}

#[derive(Debug, EnumIter, Clone)]
pub enum Tribe {
    A,
    B,
    C,
}

impl Tribe {
    pub fn random_tribe() -> Tribe {
        let mut rng = rand::thread_rng();
        return Tribe::iter().choose(&mut rng).unwrap();
    }
}
