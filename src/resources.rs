use std::collections::HashMap;

use crate::utils::*;
use bevy::prelude::*;
use crossbeam_channel::Receiver;
use hexx::{Hex, HexLayout};
use rand::rngs::StdRng;

use crate::{communication::TDCommand, components::*, config::GameConfig};

// Resources
#[derive(Debug, Resource)]
pub struct HexGrid {
    pub entities: HashMap<Hex, Entity>,
    pub paths: Option<HashMap<usize, Vec<Hex>>>,
    pub spawn_points: Vec<Hex>,
    pub layout: HexLayout,
}

// #[derive(Debug, Resource)]
// pub struct TDTimers {
//     pub enemy_spawn_rate: Timer,
//     pub tower_damaging_rate: Timer,
// }

#[derive(Debug, Resource)]
pub struct Config(pub GameConfig);

#[derive(Debug, Resource)]
pub struct TileVisuals {
    pub meshes: HashMap<MeshType, Handle<Mesh>>,
    pub materials: HashMap<MaterialType, Handle<ColorMaterial>>,
    pub damaging_materials: HashMap<DamageLevel, Handle<ColorMaterial>>,
}

#[derive(Debug, Resource)]
pub struct EnemyVisuals {
    pub meshes: HashMap<MeshType, Handle<Mesh>>,
    pub materials: HashMap<MaterialType, Handle<ColorMaterial>>,
}

#[derive(Debug, Resource)]
pub struct TowerVisuals {
    pub meshes: HashMap<MeshType, Handle<Mesh>>,
    pub materials: HashMap<TowerType, Handle<ColorMaterial>>,
}
#[derive(Debug, Resource)]
pub struct TDRng(pub StdRng);

#[derive(Debug, Resource)]
pub struct GameCommandChannel(pub Receiver<TDCommand>);

#[derive(Debug, Resource, Default)]
pub struct ScoreBoard {
    pub score: i32,
}

#[derive(Debug, Resource, Default)]
pub struct SelectedTower {
    pub selected: TowerType,
}
