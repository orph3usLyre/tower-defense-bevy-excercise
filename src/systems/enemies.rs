use crate::{
    communication::{RecalculateEnemyPaths, RefreshTowerDamage},
    components::*,
    config::EnemyConfig,
    resources::*,
    utils::*,
};
use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use hexx::{algorithms::*, Hex};
use rand::prelude::*;
use std::collections::HashMap;
use tracing::{event, Level};

pub fn handle_enemy_damage(
    mut enemies: Query<(Entity, &mut Enemy, &Transform)>,
    tiles: Query<Option<&Damaging>>,
    grid: Query<&HexGrid>,
    mut budget: Query<(&mut Budget, &mut ScoreBoard)>,
    mut timers: Query<&mut TDTimers>,
    mut commands: Commands,
    time: Res<Time>,
) {
    let mut timers = timers.single_mut();
    if timers
        .tower_damaging_rate
        .tick(time.delta())
        .just_finished()
    {
        let grid = grid.single();
        for (entity, mut enemy, transform) in enemies.iter_mut() {
            let enemy_hex_pos = grid
                .layout
                .world_pos_to_hex(transform.translation.truncate());
            if let Some(tile_entity) = grid.entities.get(&enemy_hex_pos) {
                let s = tiles.get(*tile_entity);
                match s {
                    Ok(Some(dmg)) => {
                        enemy.health = enemy.health.saturating_sub(dmg.value);
                    }
                    _ => continue,
                }
            }
            if enemy.health == 0 {
                if let Some(mut entity_commands) = commands.get_entity(entity) {
                    entity_commands.despawn();
                }
                let (mut budget, mut score_board) = budget.single_mut();
                budget.0 += enemy.value;
                // add player score
                score_board.player_score += 1;
                event!(
                    Level::INFO,
                    "Enemy died at {:?} Earned: {}",
                    enemy_hex_pos,
                    enemy.value
                );
            }
        }
    }
}

pub fn handle_enemy_movement(
    time: Res<Time>,
    path_tiles: Query<Entity, With<OnPath>>,
    grid: Query<&HexGrid>,
    paths: Query<&TDPaths>,
    config: Res<Config>,
    mut commands: Commands,
    mut enemies: Query<(Entity, &mut Moves, &mut Transform), With<Enemy>>,
    mut score_board: Query<&mut ScoreBoard>,
) {
    let mut score_board = score_board.single_mut();
    let grid = grid.single();
    for (entity, mut moves, mut transform) in enemies.iter_mut() {
        let curr_world_pos = Vec2::from((transform.translation.x, transform.translation.y));
        let curr_hex_pos = grid.layout.world_pos_to_hex(curr_world_pos);
        // if enemy still exists on tile that is no longer on path, despawn
        if path_tiles
            .get(*grid.entities.get(&curr_hex_pos).unwrap())
            .is_err()
            && curr_hex_pos != Hex::ZERO
        {
            if let Some(mut entity_commands) = commands.get_entity(entity) {
                entity_commands.despawn();
            }
        }

        let td_paths = paths.single();
        if let Some(path) = td_paths.paths.as_ref().unwrap().get(&moves.path_index.0) {
            // if index is the last in path
            if moves.path_index.1 == path.len() {
                // destroy enemies and reduce score
                if let Some(mut entity_commands) = commands.get_entity(entity) {
                    entity_commands.despawn();
                    score_board.enemy_score += 1;
                    event!(Level::DEBUG, "Enemy reached goal and despawned");
                    continue;
                }
            }
            if let Some(new_hex_pos) = path.get(moves.path_index.1 + 1) {
                moves.lerp += moves.speed * config.0.enemy_config.base_speed * time.delta_seconds();
                if moves.lerp > 1. {
                    moves.path_index.1 += 1;
                    moves.lerp -= 1.;
                }
                let new_world_pos = grid.layout.hex_to_world_pos(*new_hex_pos);
                transform.translation = curr_world_pos
                    .lerp(new_world_pos, moves.lerp)
                    .extend(transform.translation.z);
            } else {
                // this means that they got stuck on the border
                // TODO: figure out why
                moves.path_index.1 += 1;
            }
        }
    }
}

pub fn recalculate_enemy_path(
    mut commands: Commands,
    tiles: Query<(&Tile, Option<&HasTower>)>,
    mut paths: Query<&mut TDPaths>,
    on_path: Query<(Entity, &Coords), With<OnPath>>,
    grid: Query<&HexGrid>,
    mut recalculate_enemy_paths: EventReader<RecalculateEnemyPaths>,
    mut place_tower_damage: EventWriter<RefreshTowerDamage>,
) {
    if recalculate_enemy_paths.iter().last().is_some() {
        event!(
            Level::INFO,
            "Received recalculate path command, removing OnPath components"
        );
        for (entity, coords) in on_path.iter() {
            if let Some(mut entity_commands) = commands.get_entity(entity) {
                event!(Level::DEBUG, "Removed OnPath at {:?}", coords.0);
                entity_commands.remove::<OnPath>();
            }
        }
        let grid = grid.single();
        event!(Level::INFO, "Calculating enemy path");
        let mut td_paths = paths.single_mut();
        // recalculate paths
        let new_paths: HashMap<usize, Vec<Hex>> = td_paths
            .spawns
            .iter()
            .enumerate()
            .map(|(i, spawn)| {
                let path: Vec<Hex> = a_star(*spawn, Hex::ZERO, |hex| {
                    if grid.entities.contains_key(&hex) {
                        let (tile, has_tower) = tiles
                            .get(
                                *grid
                                    .entities
                                    .get(&hex)
                                    .expect("Cannot find corresponding entity to hex"),
                            )
                            .expect("Could not find entity in query");
                        if has_tower.is_some() {
                            Some(1000)
                        } else {
                            match tile.tile_type {
                                TileType::Plains => Some(0),
                                TileType::Mountain => Some(1000),
                            }
                        }
                    } else {
                        None
                    }
                })
                .unwrap();
                path.iter().for_each(|hex| {
                    if *hex != Hex::ZERO {
                        if let Some(entity) = grid.entities.get(hex) {
                            if let Some(mut entity_commands) = commands.get_entity(*entity) {
                                entity_commands.insert(OnPath);
                            }
                        }
                    }
                });
                (i, path)
            })
            .collect();
        td_paths.paths = Some(new_paths);
        recalculate_enemy_paths.clear();
        place_tower_damage.send(RefreshTowerDamage);
    }
}

pub fn spawn_enemies(
    mut commands: Commands,
    time: Res<Time>,
    board: Query<Entity, With<TDBoard>>,
    paths: Query<&mut TDPaths>,
    grid: Query<&HexGrid>,
    config: Res<Config>,
    mut timers: Query<&mut TDTimers>,
    enemy_visuals: Res<EnemyVisuals>,
    mut rng: ResMut<TDRng>,
) {
    let mut timers = timers.single_mut();
    let grid = grid.single();
    if timers.enemy_spawn_rate.tick(time.delta()).just_finished() {
        let board_entity = board.single();
        let paths = paths.single();
        if let Some(paths) = paths.paths.as_ref() {
            let path = paths.iter().choose(&mut rng.0).unwrap();
            let spawn_location = path.1.first().unwrap();
            let Vec2 { x, y } = grid.layout.hex_to_world_pos(*spawn_location);
            let EnemyConfig {
                base_speed,
                min_max_health,
                ..
            } = config.0.enemy_config;

            let health: u32 = rng.0.gen_range(min_max_health.0..=min_max_health.1);
            let scale = health as f32 / (min_max_health.1 - min_max_health.0) as f32;
            let speed = (1.0 - (health as f32 / (min_max_health.1 - min_max_health.0) as f32))
                .clamp(0.1, 1.)
                * base_speed;
            let value = health;
            commands
                .spawn((
                    MaterialMesh2dBundle {
                        mesh: enemy_visuals
                            .meshes
                            .get(&MeshType::Enemy)
                            .unwrap()
                            .clone()
                            .into(),
                        material: enemy_visuals
                            .materials
                            .get(&MaterialType::Enemy)
                            .unwrap()
                            .clone(),
                        transform: Transform::from_translation(Vec3::from((x, y, 5.)))
                            .with_scale(Vec3::splat(scale)),
                        ..default()
                    },
                    Enemy { health, value },
                    Moves {
                        lerp: 0.,
                        path_index: (*path.0, 0),
                        speed,
                    },
                ))
                .set_parent(board_entity);
        }
    }
}
