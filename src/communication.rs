use crate::components::TowerType;
use hexx::Hex;

#[derive(Debug, Copy, Clone)]
pub struct CreateTower {
    pub tower_type: TowerType,
    pub hex_pos: Hex,
}

#[derive(Debug)]
pub struct GameOver;

#[derive(Debug, Copy, Clone)]
pub struct ToggleTile {
    pub hex_pos: Hex,
}
#[derive(Debug)]
pub struct RefreshTowerDamage;

#[derive(Debug)]
pub struct Restart;

#[derive(Debug)]
pub struct RecalculateEnemyPaths;

// outside communication
#[derive(Debug)]
pub enum TDCommand {
    Toggle(ToggleTile),
    Restart(Restart),
    Tower(CreateTower),
}

pub fn parse_command(input: &str) -> Option<TDCommand> {
    let split: Vec<_> = input.split_whitespace().collect();
    let command = split.first()?;
    match *command {
        "reset" => Some(TDCommand::Restart(Restart)),
        "toggle" => {
            let values = split.get(1)?;
            let (x, y) = values.split_once(',')?;
            let (x, y) = (x.parse().ok()?, y.parse().ok()?);
            Some(TDCommand::Toggle(ToggleTile {
                hex_pos: Hex { x, y },
            }))
        }
        "tower" => {
            let values = split.get(1)?;
            let (x, y) = values.split_once(',')?;
            let (x, y) = (x.parse().ok()?, y.parse().ok()?);
            let tower_type = match *split.get(2)? {
                "s" => TowerType::Small,
                "m" => TowerType::Medium,
                "l" => TowerType::Large,
                _ => return None,
            };

            Some(TDCommand::Tower(CreateTower {
                hex_pos: Hex { x, y },
                tower_type,
            }))
        }
        _ => None,
    }
}
