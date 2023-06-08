use std::collections::HashMap;

use crate::utils::*;
use bevy::{
    prelude::{Component, Entity},
    time::Timer,
};
use hexx::{Hex, HexLayout};
use serde::{Deserialize, Serialize};

// Camera
#[derive(Debug, Component)]
pub struct TDCamera;

// Board
#[derive(Debug, Component)]
pub struct Budget(pub u32);

#[derive(Debug, Component)]
pub struct HexGrid {
    pub entities: HashMap<Hex, Entity>,
    pub layout: HexLayout,
}

#[derive(Debug, Component)]
pub struct TDBoard;

#[derive(Debug, Component)]
pub struct TDTimers {
    pub enemy_spawn_rate: Timer,
    pub tower_damaging_rate: Timer,
    pub game_over_timer: Timer,
}

#[derive(Debug, Component, Default)]
pub struct ScoreBoard {
    pub player_score: u32,
    pub enemy_score: u32,
}

#[derive(Debug, Component, Default)]
pub struct GameTimer(pub Timer);

#[derive(Debug, Component)]
pub struct TDPaths {
    pub spawns: Vec<Hex>,
    pub paths: Option<HashMap<usize, Vec<Hex>>>,
}

// Text
#[derive(Debug, Component)]
pub struct GameOverText;

// Tiles
#[derive(Debug, Component)]
pub struct Coords(pub Hex);

#[derive(Debug, Component)]
pub struct HasTower;

#[derive(Debug, Component)]
pub struct IsGoal;

#[derive(Debug, Component)]
pub struct IsSpawn;

#[derive(Debug, Component)]
pub struct OnPath;

#[derive(Debug, Component)]
pub struct Refresh;

#[derive(Debug, Component)]
pub struct Damaging {
    pub value: u32,
}

#[derive(Debug, Component)]
pub struct DamagingBase;

#[derive(Debug, PartialEq, Eq, Hash, Component)]
pub struct Tile {
    pub tile_type: TileType,
    pub is_cursor: bool,
}

#[derive(Debug, PartialEq, Eq, Hash, Component)]
pub enum TileType {
    Plains,
    Mountain,
}

impl TileType {
    pub fn material_type(&self) -> MaterialType {
        match self {
            TileType::Plains => MaterialType::Plains,
            TileType::Mountain => MaterialType::Mountain,
        }
    }
}

// Enemies
#[derive(Debug, Component)]
pub struct Enemy {
    pub health: u32,
    pub value: u32,
}
#[derive(Debug, Component)]
pub struct Moves {
    pub path_index: (usize, usize),
    pub lerp: f32,
    pub speed: f32,
}

// Towers
#[derive(Debug, Component)]
pub struct Tower {
    pub tower_type: TowerType,
    pub cost: u32,
}

#[derive(
    Debug, Default, PartialEq, Eq, Hash, Copy, Clone, PartialOrd, Ord, Serialize, Deserialize,
)]
pub enum TowerType {
    #[default]
    Small,
    Medium,
    Large,
}

impl TowerType {
    pub fn range(&self) -> u32 {
        match self {
            TowerType::Small => 1,
            TowerType::Medium => 2,
            TowerType::Large => 3,
        }
    }
    pub fn damage(&self) -> u32 {
        match self {
            TowerType::Small => 1,
            TowerType::Medium => 2,
            TowerType::Large => 3,
        }
    }
}
