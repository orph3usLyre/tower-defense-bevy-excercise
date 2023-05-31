use bevy::prelude::Component;
use hexx::Hex;

#[derive(Debug, Component)]
pub struct TDBoard;

#[derive(Debug, Component)]
pub struct OnPath;

#[derive(Debug, Component)]
pub struct TDState {
    pub restart: bool,
    pub recalculate_enemy_paths: bool,
}

#[derive(Debug, Component)]
pub struct IsGoal;

#[derive(Debug, Component)]
pub struct IsCursorTarget;

#[derive(Debug, Component)]
pub struct IsSpawn;

#[derive(Debug, Component)]
pub struct CursorTarget;

#[derive(Debug, Component)]
pub struct Coords(pub Hex);

#[derive(Debug, PartialEq, Eq, Hash, Component)]
pub enum TileType {
    Plains,
    Mountain,
    Goal,
}

impl TileType {
    pub fn material_type(&self) -> MaterialType {
        match self {
            TileType::Plains => MaterialType::Plains,
            TileType::Mountain => MaterialType::Mountain,
            TileType::Goal => MaterialType::Goal,
        }
    }
}

#[derive(Debug, Component)]
pub struct Enemy {
    pub health: u32,
}
#[derive(Debug, Component)]
pub struct Moves {
    pub path_index: usize,
    pub lerp: f32,
}

#[derive(Debug, Component)]
pub struct Tower {
    pub tower_type: TowerType,
}

#[derive(Debug, Default, PartialEq, Eq, Hash, Copy, Clone)]
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

#[derive(Debug, Component)]
pub struct Damaging {
    pub value: u32,
}
#[derive(Debug, Component)]
pub struct DamagingBase;

// helpers
#[derive(Debug, PartialEq, Eq, Hash)]
pub enum MaterialType {
    Plains,
    Mountain,
    Path,
    Goal,
    Spawn,
    Target,
    Enemy,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum MeshType {
    Hex,
    Enemy,
    Tower,
}
#[derive(Debug, PartialEq, Eq, Hash)]
pub enum DamageLevel {
    Low,
    Medium,
    High,
}

impl DamageLevel {
    pub fn get_level(damage: u32) -> Self {
        match damage {
            0..=3 => Self::Low,
            4..=7 => Self::Medium,
            _ => Self::High,
        }
    }
}
