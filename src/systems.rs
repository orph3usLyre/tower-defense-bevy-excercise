use std::collections::HashMap;

use bevy::sprite::MaterialMesh2dBundle;
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::{egui, EguiContexts};
use hexx::{algorithms::*, Hex};
use rand::seq::IteratorRandom;
use rand::Rng;
use tracing::{event, Level};

use crate::components::{
    Enemy, IsCursorTarget, IsGoal, IsSpawn, MaterialType, MeshType, Moves, OnPath, TDBoard,
    TDState, TileType,
};
use crate::resources::{GameCommandChannel, HexGrid, ScoreBoard};
use crate::{utils::*, TDCommand};

const SPEED: f32 = 5.;

// Systems
pub fn show_ui(
    mut contexts: EguiContexts,
    tiles: Query<Entity, With<IsCursorTarget>>,
    mut board_state: Query<&mut TDState>,
    grid: Res<HexGrid>,
    score_board: Res<ScoreBoard>,
) {
    egui::Window::new("Info").show(contexts.ctx_mut(), |ui| {
        ui.label("Selected tile");
        if let Ok(cursor_entity) = tiles.get_single() {
            if let Some((hex, _)) = grid
                .entities
                .iter()
                .find(|(_, entity)| entity == &&cursor_entity)
            {
                ui.label(format!("Coord: x: {}, y: {}", hex.x(), hex.y()));
            }
        } else {
            ui.label(format!("None selected",));
        }

        ui.horizontal(|ui| {
            if ui.button("Restart Board").clicked() {
                let mut board_state = board_state.single_mut();
                board_state.restart = true;
            }
        });
        ui.label("Score Board");
        ui.horizontal(|ui| {
            ui.label(format!("Score: {}", score_board.score));
        });
    });
}

pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

pub fn setup_grid(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>,
) {
    generate_board(&mut commands, meshes, materials);
}

// Input interaction
pub fn handle_input(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut tiles: Query<(Entity, &mut TileType, Option<&OnPath>, Option<&IsGoal>)>,
    mut current: Local<Hex>,
    mut commands: Commands,
    grid: ResMut<HexGrid>,
    buttons: Res<Input<MouseButton>>,
    mut board_state: Query<&mut TDState>,
    mut previous_cursor: Local<Hex>,
) {
    let window = windows.single();
    if let Some(pos) = window.cursor_position() {
        let pos = pos - Vec2::new(window.width(), window.height()) / 2.0;
        let hex_pos = grid.layout.world_pos_to_hex(pos);

        // change tiles according to buttons
        if buttons.just_pressed(MouseButton::Left) {
            event!(Level::INFO, "Left click pressed");
            if let Some(entity) = grid.entities.get(&hex_pos) {
                // functionality for left button
                let (_, mut tile_type, on_path, _is_goal) = tiles.get_mut(*entity).unwrap();
                if let Some(_) = on_path {
                    *tile_type = TileType::Mountain;
                    let mut board_state = board_state.single_mut();
                    board_state.recalculate_enemy_paths = true;
                } else if *tile_type == TileType::Mountain {
                    *tile_type = TileType::Plains;
                } else if *tile_type == TileType::Plains {
                    *tile_type = TileType::Mountain;
                }
            }
        }
        // functionality for right button press
        if buttons.just_pressed(MouseButton::Right) {}

        if hex_pos == *current {
            return;
        }
        *current = hex_pos;

        // add cursor marker component
        if let Some(cursor_entity) = grid.entities.get(&hex_pos) {
            commands.entity(*cursor_entity).insert(IsCursorTarget);
            if let Some(previous_cursor) = grid.entities.get(&previous_cursor) {
                commands.entity(*previous_cursor).remove::<IsCursorTarget>();
            }
        } else {
            if let Some(previous_cursor) = grid.entities.get(&previous_cursor) {
                commands.entity(*previous_cursor).remove::<IsCursorTarget>();
            }
        }
        *previous_cursor = hex_pos;
    }
}

pub fn handle_color_change(
    mut commands: Commands,
    mut tile_entities: Query<
        (Entity, &TileType, Option<&IsCursorTarget>, Option<&OnPath>),
        (Without<IsGoal>, Without<IsSpawn>),
    >,
    grid: Res<HexGrid>,
) {
    for (entity, tile_type, is_cursor, on_path) in tile_entities.iter_mut() {
        if let Some(_) = is_cursor {
            commands
                .entity(entity)
                .insert(grid.materials.get(&MaterialType::Target).unwrap().clone());
        } else if let Some(_) = on_path {
            commands
                .entity(entity)
                .insert(grid.materials.get(&MaterialType::Path).unwrap().clone());

            // event!(Level::INFO, "On path material added");
        } else {
            let material = match tile_type {
                TileType::Plains => MaterialType::Plains,
                TileType::Mountain => MaterialType::Mountain,
                TileType::Goal => MaterialType::Goal,
            };
            commands
                .entity(entity)
                .insert(grid.materials.get(&material).unwrap().clone());
        }
    }
}

pub fn recalculate_enemy_path(
    mut commands: Commands,
    mut board_state: Query<&mut TDState>,
    tiles: Query<&TileType>,
    on_path: Query<Entity, With<OnPath>>,
    mut grid: ResMut<HexGrid>,
) {
    if !board_state.single().recalculate_enemy_paths {
        return;
    }
    for entity in on_path.iter() {
        commands.entity(entity).remove::<OnPath>();
    }

    event!(Level::INFO, "Calculating enemy path");

    let new_paths: HashMap<usize, Vec<Hex>> = grid
        .spawn_points
        .iter()
        .enumerate()
        .map(|(i, spawn)| {
            let path: Vec<Hex> = a_star(*spawn, Hex::ZERO, |hex| {
                if grid.entities.contains_key(&hex) {
                    let entity = grid.entities.get(&hex).unwrap();
                    match tiles.get(*entity).unwrap() {
                        TileType::Plains => Some(0),
                        TileType::Mountain => Some(1000),
                        TileType::Goal => Some(0),
                    }
                } else {
                    None
                }
            })
            .unwrap();
            path.iter().for_each(|hex| {
                if *hex != Hex::ZERO {
                    let entity = grid.entities.get(&hex).copied().unwrap();
                    let _ = commands.entity(entity).insert(OnPath).id();
                }
            });
            (i, path)
        })
        .collect();
    grid.paths = Some(new_paths);

    let mut board_state = board_state.single_mut();
    board_state.recalculate_enemy_paths = false;
}

pub fn restart(
    mut commands: Commands,
    mut board: Query<(Entity, &mut TDState), With<TDBoard>>,
    mut score_board: ResMut<ScoreBoard>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>,
) {
    let (board_entity, mut board_state) = board.single_mut();
    if !board_state.restart {
        return;
    }
    event!(Level::INFO, "Resetting board");
    destroy_board(&mut commands, board_entity);
    board_state.restart = false;
    score_board.score = 0;
    generate_board(&mut commands, meshes, materials);
}

pub fn spawn_enemies(
    mut commands: Commands,
    time: Res<Time>,
    board: Query<Entity, With<TDBoard>>,
    mut grid: ResMut<HexGrid>,
) {
    let mut rng = rand::thread_rng();
    let scale: f32 = rng.gen();
    if grid.enemy_spawn_rate.tick(time.delta()).just_finished() {
        let board_entity = board.single();
        let spawn_location = grid.spawn_points.iter().choose(&mut rng).unwrap();
        let Vec2 { x, y } = grid.layout.hex_to_world_pos(*spawn_location);
        commands
            .spawn((
                MaterialMesh2dBundle {
                    mesh: grid.meshes.get(&MeshType::Enemy).unwrap().clone().into(),
                    material: grid
                        .materials
                        .get(&MaterialType::Enemy)
                        .unwrap()
                        .clone()
                        .into(),
                    transform: Transform::from_translation(Vec3::from((x, y, 1.)))
                        .with_scale(Vec3::splat(scale)),
                    ..default()
                },
                Enemy { health: 10 },
                Moves {
                    lerp: 0.,
                    path_index: 0,
                },
            ))
            .set_parent(board_entity);
    }
}

pub fn handle_enemy_movement(
    time: Res<Time>,
    // path_tiles: Query<Entity, With<OnPath>>,
    grid: Res<HexGrid>,
    mut commands: Commands,
    mut enemies: Query<(Entity, &mut Moves, &mut Transform)>,
    mut score_board: ResMut<ScoreBoard>,
) {
    for (entity, mut moves, mut transform) in enemies.iter_mut() {
        let curr_world_pos = Vec2::from((transform.translation.x, transform.translation.y));
        let curr_hex_pos = grid.layout.world_pos_to_hex(curr_world_pos);
        let curr_center = grid.layout.hex_to_world_pos(curr_hex_pos);
        if curr_hex_pos == Hex::ZERO {
            // destroy enemies and reduce score

            commands.entity(entity).despawn();
            // add value to enemies
            score_board.score += 1;
            continue;
        }

        let path = grid
            .paths
            .as_ref()
            .unwrap()
            .values()
            .find(|hv| hv.contains(&curr_hex_pos));

        if let Some(path) = path {
            if moves.path_index + 1 >= path.len() {
                continue;
            } else {
                moves.lerp += SPEED * time.delta_seconds();
                if moves.lerp > 1. {
                    moves.path_index += 1;
                    moves.lerp -= 1.;
                }
                // let curr_hex = path.get(index).unwrap();
                // let curr_hex_pos = path.get(moves.path_index).unwrap();
                // let curr_world_pos = grid.layout.hex_to_world_pos(*curr_hex_pos); // center of target
                let new_hex_pos = path.get(moves.path_index + 1).unwrap();
                let new_world_pos = grid.layout.hex_to_world_pos(*new_hex_pos); // center of target
                                                                                // position

                //
                transform.translation = curr_center.lerp(new_world_pos, moves.lerp).extend(0.);
            }
        }
    }
}

pub fn execute_outside_commands(
    mut command_channel: EventReader<TDCommand>,
    mut board: Query<&mut TDState>,
) {
    for command in command_channel.iter() {
        event!(Level::INFO, "Received command: {:#?}", command);
        match command {
            TDCommand::Toggle => {
                // TODO: should toggle a tile
            }
            TDCommand::Reset => {
                let mut board_state = board.single_mut();
                board_state.restart = true;
                return;
            }
        }
    }
}

pub fn event_dispatch(mut command_send: EventWriter<TDCommand>, channel: Res<GameCommandChannel>) {
    if channel.0.is_empty() {
        return;
    }
    for instruction in channel.0.iter() {
        event!(
            Level::INFO,
            "Command sent (event_dispatch): {:#?}",
            instruction
        );
        command_send.send(instruction);
    }
}
