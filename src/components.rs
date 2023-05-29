use bevy::prelude::Component;

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

#[derive(Debug, PartialEq, Eq, Hash, Component)]
pub enum TileType {
    Plains,
    Mountain,
    Goal,
}

#[derive(Debug, Component)]
pub struct Enemy {
    pub health: i32,
}
#[derive(Debug, Component)]
pub struct Moves {
    pub path_index: usize,
    pub lerp: f32,
}
// helpers
//
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
}
