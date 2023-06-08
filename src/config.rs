use std::{collections::BTreeMap, io::Write};

use hexx::Vec2;
use serde::{Deserialize, Serialize};
use tracing::{event, Level};

use crate::components::TowerType;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GameConfig {
    pub map_radius: u32,
    pub hex_size: Vec2,
    pub seed: Option<u64>,
    pub zoom_speed: f32,
    pub starting_budget: u32,
    pub tower_config: TowersConfig,
    pub enemy_config: EnemyConfig,
    pub game_length: f32,
    pub game_over_timer_length: f32,
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
            game_length: 60.,
            game_over_timer_length: 5.,
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

    pub fn export(&self, path: &str) -> Result<(), &'static str> {
        let config_as_str =
            serde_yaml::to_string(&self).map_err(|_| "Unable to serialize config")?;
        let mut config_file = std::fs::File::options()
            .truncate(true)
            .write(true)
            .open(path)
            .map_err(|_| "Unable to open file")?;
        config_file
            .write_all(config_as_str.as_bytes())
            .map_err(|_| "Unable to write to file")?;

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TowersConfig {
    pub tower_type: BTreeMap<TowerType, TowerConfig>,
    pub damaging_rate: f32,
    pub tower_damage_alpha: f32,
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
            tower_type: BTreeMap::from([
                (TowerType::Small, small),
                (TowerType::Medium, medium),
                (TowerType::Large, large),
            ]),
            damaging_rate: 0.3,
            tower_damage_alpha: 0.7,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct TowerConfig {
    pub cost: u32,
    pub scale: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct EnemyConfig {
    pub enemy_spawn_rate: f32,
    pub base_speed: f32,
    pub min_max_health: (u32, u32),
}

impl Default for EnemyConfig {
    fn default() -> Self {
        Self {
            base_speed: 2.,
            enemy_spawn_rate: 0.2,
            min_max_health: (5, 20),
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
