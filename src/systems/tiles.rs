use crate::communication::*;
use crate::components::*;
use crate::resources::*;
use crate::AppState;
use bevy::prelude::*;
use hexx::{DiagonalDirection, Hex, HexLayout};
use rand::prelude::*;
use std::collections::HashMap;
use tracing::event;
use tracing::Level;

pub fn handle_removed_paths(
    mut commands: Commands,
    mut removed: RemovedComponents<OnPath>,
    damaging_tiles: Query<&Children, &Damaging>,
) {
    // remove the damaging base
    for removed_path in removed.iter() {
        if let Ok(children) = damaging_tiles.get(removed_path) {
            for child in children.iter() {
                if let Some(mut entity_commands) = commands.get_entity(*child) {
                    entity_commands.despawn();
                }
            }
            commands.entity(removed_path).remove::<Damaging>();
        }
        if let Some(mut entity_commands) = commands.get_entity(removed_path) {
            entity_commands.insert(Refresh);
            event!(Level::DEBUG, "Refreshed tile with removed path");
        }
    }
}

pub fn spawn_board_and_tiles(
    mut commands: Commands,
    mut rng: ResMut<TDRng>,
    config: Res<Config>,
    mut recalculate_paths: EventWriter<RecalculateEnemyPaths>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let layout = HexLayout {
        hex_size: config.0.hex_size,
        ..default()
    };
    // setup timers
    event!(Level::INFO, "TDTimers");
    let td_timers = TDTimers {
        enemy_spawn_rate: Timer::from_seconds(
            config.0.enemy_config.enemy_spawn_rate,
            TimerMode::Repeating,
        ),
        tower_damaging_rate: Timer::from_seconds(
            config.0.tower_config.damaging_rate,
            TimerMode::Repeating,
        ),
    };
    let budget = Budget(config.0.starting_budget);
    // board setup
    let board = commands
        .spawn((
            SpatialBundle {
                visibility: Visibility::Visible,
                ..Default::default()
            },
            TDBoard,
            td_timers,
            budget,
        ))
        .id();

    // find spawn locations
    // TODO: makes this a component
    let spawns: Vec<Hex> = DiagonalDirection::iter()
        .map(|d| {
            Hex::ZERO
                .ring_edge(config.0.map_radius, d)
                .choose(&mut rng.0)
                .unwrap()
        })
        .collect();
    // create tile entities
    let entities: HashMap<Hex, Entity> = Hex::ZERO
        .spiral_range(0..=config.0.map_radius)
        .enumerate()
        .map(|(_i, coord)| {
            let random = rng.0.gen_range(0..=3); // TODO: make this percentage based in config
            let tile_type = match random {
                0..=2 => TileType::Plains,
                _ => TileType::Mountain,
            };
            let mut child = commands.spawn((
                Tile {
                    tile_type,
                    is_cursor: false,
                },
                Coords(coord),
            ));

            child.set_parent(board);
            if coord == Hex::ZERO {
                child.insert(IsGoal);
            } else if spawns.contains(&coord) {
                child.insert(IsSpawn);
            }
            (coord, child.id())
        })
        .collect();

    commands.insert_resource(HexGrid {
        entities,
        layout,
        paths: None,
        spawn_points: spawns,
    });
    // recalculate paths after setup
    recalculate_paths.send(RecalculateEnemyPaths);
    // move app state in game
    next_state.set(AppState::InGame);
}

pub fn toggle_tile(
    mut toggle_tiles: EventReader<ToggleTile>,
    mut tiles: Query<(&mut Tile, Option<&OnPath>)>,
    mut recalculate_enemy_paths: EventWriter<RecalculateEnemyPaths>,
    grid: Res<HexGrid>,
) {
    for t in toggle_tiles.iter() {
        if let Ok((mut tile, on_path)) = tiles.get_mut(*grid.entities.get(&t.hex_pos).unwrap()) {
            event!(Level::INFO, "Toggling tile at {:?}", t.hex_pos);
            match tile.tile_type {
                TileType::Plains => tile.tile_type = TileType::Mountain,
                TileType::Mountain => tile.tile_type = TileType::Plains,
            }
            if on_path.is_some() {
                recalculate_enemy_paths.send(RecalculateEnemyPaths);
            }
        }
    }
}
