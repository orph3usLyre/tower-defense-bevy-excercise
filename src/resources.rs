use std::collections::HashMap;

use bevy::prelude::*;
use crossbeam_channel::Receiver;
use hexx::{Hex, HexLayout};

use crate::{
    communication::TDCommand,
    components::{MaterialType, MeshType},
};

// Resources
#[derive(Debug, Resource)]
pub struct HexGrid {
    pub entities: HashMap<Hex, Entity>,
    pub paths: Option<HashMap<usize, Vec<Hex>>>,
    pub spawn_points: Vec<Hex>,
    // pub reachable_entities: HashSet<Entity>,
    pub layout: HexLayout,
    pub materials: HashMap<MaterialType, Handle<ColorMaterial>>,
    pub meshes: HashMap<MeshType, Handle<Mesh>>,
    pub enemy_spawn_rate: Timer,
}

#[derive(Debug, Resource)]
pub struct GameCommandChannel(pub Receiver<TDCommand>);

#[derive(Debug, Resource, Default)]
pub struct ScoreBoard {
    pub score: i32,
}
