use std::collections::HashMap;

use hexx::Vec2;
use serde::{Deserialize, Serialize};
use tracing::{event, Level};

use crate::components::TowerType;

#[derive(Debug, Serialize, Deserialize)]
pub struct GameConfig {
    pub map_radius: u32,
    pub hex_size: Vec2,
    pub seed: Option<u64>,
    pub zoom_speed: f32,
    pub starting_budget: u32,
    pub tower_config: TowersConfig,
    pub enemy_config: EnemyConfig,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            map_radius: 20,
            hex_size: Vec2::splat(10.),
            seed: None,
            zoom_speed: 1.,
            starting_budget: 50,
            tower_config: TowersConfig::default(),
            enemy_config: EnemyConfig::default(),
        }
    }
}

impl GameConfig {
    pub fn load(path: &str) -> Self {
        let config_yaml: String = std::fs::read_to_string(path).unwrap();
        if let Ok(config) = serde_yaml::from_str(&config_yaml) {
            config
        } else {
            event!(Level::WARN, "Unable to load config from file");
            GameConfig::default()
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TowersConfig {
    pub config_type: HashMap<TowerType, TowerConfig>,
    pub damaging_rate: f32,
}

impl Default for TowersConfig {
    fn default() -> Self {
        let small = TowerConfig {
            cost: 8,
            scale: 0.8,
        };
        let medium = TowerConfig {
            cost: 20,
            scale: 1.,
        };
        let large = TowerConfig {
            cost: 50,
            scale: 1.5,
        };
        Self {
            config_type: HashMap::from([
                (TowerType::Small, small),
                (TowerType::Medium, medium),
                (TowerType::Large, large),
            ]),
            damaging_rate: 0.3,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TowerConfig {
    pub cost: u32,
    pub scale: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnemyConfig {
    pub enemy_spawn_rate: f32,
    pub base_speed: f32,
}

impl Default for EnemyConfig {
    fn default() -> Self {
        Self {
            base_speed: 0.5,
            enemy_spawn_rate: 0.2,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::config::GameConfig;

    #[test]
    fn serialize() {
        let config = GameConfig::default();
        println!("{}", serde_yaml::to_string(&config).unwrap());
    }
}
