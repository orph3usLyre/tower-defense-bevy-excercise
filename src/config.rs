use bevy::prelude::Resource;
use hexx::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Resource)]
pub struct GameConfig {
    pub map_radius: u32,
    pub hex_size: Vec2,
    pub seed: Option<u64>,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            map_radius: 20,
            hex_size: Vec2::splat(10.),
            seed: None,
        }
    }
}
