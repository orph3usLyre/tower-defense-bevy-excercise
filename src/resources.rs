use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use crossbeam_channel::Receiver;
use hexx::{Hex, HexLayout};

use crate::communication::TDCommand;

// Resources
#[derive(Debug, Resource)]
pub struct HexGrid {
    pub entities: HashMap<Hex, (Option<u32>, Entity)>,
    pub reachable_entities: HashSet<Entity>,
    pub layout: HexLayout,
}

#[derive(Debug, Resource)]
pub struct GameCommandChannel(pub Receiver<TDCommand>);
