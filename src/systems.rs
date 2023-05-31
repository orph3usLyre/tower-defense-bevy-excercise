use std::collections::HashMap;

use crate::components::*;
use crate::config::GameConfig;
use crate::resources::{
    EnemyVisuals, GameCommandChannel, HexGrid, ScoreBoard, SelectedTower, TDRng, TileVisuals,
    TowerVisuals,
};
use crate::{utils::*, TDCommand};
use bevy::sprite::MaterialMesh2dBundle;
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::{egui, EguiContexts};
use hexx::{algorithms::*, DiagonalDirection, Hex, HexLayout};
use rand::rngs::StdRng;
use rand::seq::IteratorRandom;
use rand::{Rng, SeedableRng};
use tracing::{event, Level};

const SPEED: f32 = 5.;

// Systems
pub fn show_ui(
    mut contexts: EguiContexts,
    tiles: Query<(&Coords, Option<&Damaging>), With<IsCursorTarget>>,
    mut board_state: Query<&mut TDState>,
    score_board: Res<ScoreBoard>,
    mut selected_tower: ResMut<SelectedTower>,
) {
    egui::Window::new("Info").show(contexts.ctx_mut(), |ui| {
        ui.label("Selected tile");
        if let Ok((hex, damaging)) = tiles.get_single() {
            ui.label(format!("Coord: x: {}, y: {}", hex.0.x(), hex.0.y()));
            let text = if let Some(dmg) = damaging {
                dmg.value
            } else {
                0
            };
            ui.label(format!("Damaging: {}", text));
        } else {
            ui.label("None selected".to_string());
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

    egui::Window::new("Towers").show(contexts.ctx_mut(), |ui| {
        ui.label("Selected tower");
        egui::ComboBox::from_label("Select one!")
            .selected_text(format!("{:?}", selected_tower.selected))
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut selected_tower.selected,
                    TowerType::Small,
                    "Small tower",
                );
                ui.selectable_value(
                    &mut selected_tower.selected,
                    TowerType::Medium,
                    "Medium tower",
                );
                ui.selectable_value(
                    &mut selected_tower.selected,
                    TowerType::Large,
                    "Large tower",
                );
            });
    });
}

pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

// Input interaction
#[allow(clippy::too_many_arguments)]
pub fn handle_input(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut tiles: Query<(Entity, &mut TileType, Option<&OnPath>, Option<&IsGoal>)>,
    mut current: Local<Hex>,
    mut commands: Commands,
    grid: ResMut<HexGrid>,
    buttons: Res<Input<MouseButton>>,
    mut board_state: Query<&mut TDState>,
    mut previous_cursor: Local<Hex>,
    tower_visuals: Res<TowerVisuals>,
    board_entity: Query<Entity, With<TDBoard>>,
    selected_tower: Res<SelectedTower>,
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
                if on_path.is_some() {
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
        if buttons.just_pressed(MouseButton::Right) {
            if let Some(entity) = grid.entities.get(&hex_pos) {
                if let Ok((_, _tile_type, on_path, is_goal)) = tiles.get(*entity) {
                    if is_goal.is_some() || on_path.is_some() {
                        return;
                    } else {
                        // spawn tower
                        let tower_type = selected_tower.selected;
                        let scale = match tower_type {
                            TowerType::Small => 0.8,
                            TowerType::Medium => 1.,
                            TowerType::Large => 1.5,
                        };
                        commands
                            .spawn((
                                MaterialMesh2dBundle {
                                    mesh: tower_visuals
                                        .meshes
                                        .get(&MeshType::Tower)
                                        .unwrap()
                                        .clone()
                                        .into(),
                                    material: tower_visuals
                                        .materials
                                        .get(&tower_type)
                                        .unwrap()
                                        .clone(),
                                    transform: Transform::from_translation(
                                        grid.layout.hex_to_world_pos(hex_pos).extend(2.),
                                    )
                                    .with_scale(Vec3::splat(scale)),
                                    ..default()
                                },
                                Tower { tower_type },
                                Coords(hex_pos),
                            ))
                            .set_parent(board_entity.single());
                    }
                }
            }
        }

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
        } else if let Some(previous_cursor) = grid.entities.get(&previous_cursor) {
            commands.entity(*previous_cursor).remove::<IsCursorTarget>();
        }
        *previous_cursor = hex_pos;
    }
}
#[allow(clippy::type_complexity)]
pub fn handle_color_change(
    mut commands: Commands,
    mut tile_entities: Query<
        (
            Entity,
            &TileType,
            Option<&IsCursorTarget>,
            Option<&OnPath>,
            Option<&Damaging>,
            Option<&Children>,
        ),
        (Without<IsGoal>, Without<IsSpawn>),
    >,
    damaging_entitys: Query<Entity, With<DamagingBase>>,
    tile_visuals: Res<TileVisuals>,
) {
    for (entity, tile_type, is_cursor, on_path, damaging, children) in tile_entities.iter_mut() {
        if is_cursor.is_some() {
            commands.entity(entity).insert(
                tile_visuals
                    .materials
                    .get(&MaterialType::Target)
                    .unwrap()
                    .clone(),
            );
        } else if on_path.is_some() {
            commands.entity(entity).insert(
                tile_visuals
                    .materials
                    .get(&MaterialType::Path)
                    .unwrap()
                    .clone(),
            );
        } else {
            let material = match tile_type {
                TileType::Plains => MaterialType::Plains,
                TileType::Mountain => MaterialType::Mountain,
                TileType::Goal => MaterialType::Goal,
            };
            commands
                .entity(entity)
                .insert(tile_visuals.materials.get(&material).unwrap().clone());
        }
        // if tile is damaging and has children (which it should),
        // we get the child entity and insert the damage material based on the damaging
        // value of the tile
        if let Some(damaging) = damaging {
            if let Some(children) = children {
                let child = children.get(0).unwrap();
                if let Ok(child_entity) = damaging_entitys.get(*child) {
                    let damage_level = DamageLevel::get_level(damaging.value);
                    let material = tile_visuals
                        .damaging_materials
                        .get(&damage_level)
                        .unwrap()
                        .clone();
                    commands.entity(child_entity).insert(material);
                }
            }
        }
    }
}

pub fn recalculate_enemy_path(
    mut commands: Commands,
    mut board_state: Query<&mut TDState>,
    tiles: Query<(Entity, &TileType, &Coords)>,
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
                    let (_, tile_type, _) = tiles
                        .get(
                            *grid
                                .entities
                                .get(&hex)
                                .expect("Cannot find corresponding entity to hex"),
                        )
                        .expect("Could not find entity in query");
                    match tile_type {
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
                    if let Some(entity) = grid.entities.get(hex) {
                        let _ = commands.entity(*entity).insert(OnPath).id();
                    }
                }
            });
            (i, path)
        })
        .collect();
    grid.paths = Some(new_paths);

    let mut board_state = board_state.single_mut();
    board_state.recalculate_enemy_paths = false;
}
pub fn restart(mut commands: Commands, board: Query<Entity, With<TDBoard>>) {
    let board_entity = board.single();
    event!(Level::INFO, "Resetting board");
    destroy_board(&mut commands, board_entity);
}

pub fn spawn_enemies(
    mut commands: Commands,
    time: Res<Time>,
    board: Query<Entity, With<TDBoard>>,
    mut grid: ResMut<HexGrid>,
    enemy_visuals: Res<EnemyVisuals>,
    mut rng: ResMut<TDRng>,
) {
    let scale: f32 = rng.0.gen();
    if grid.enemy_spawn_rate.tick(time.delta()).just_finished() {
        let board_entity = board.single();
        let spawn_location = grid.spawn_points.iter().choose(&mut rng.0).unwrap();
        let Vec2 { x, y } = grid.layout.hex_to_world_pos(*spawn_location);
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
                    transform: Transform::from_translation(Vec3::from((x, y, 2.)))
                        .with_scale(Vec3::splat(scale)),
                    ..default()
                },
                Enemy { health: 10 },
                Moves {
                    lerp: 0.,
                    path_index: 0,
                },
                Coords(*spawn_location),
            ))
            .set_parent(board_entity);
    }
}

pub fn handle_enemy_movement(
    time: Res<Time>,
    path_tiles: Query<Entity, With<OnPath>>,
    grid: Res<HexGrid>,
    mut commands: Commands,
    mut enemies: Query<(Entity, &mut Moves, &mut Transform), With<Enemy>>,
    mut score_board: ResMut<ScoreBoard>,
) {
    for (entity, mut moves, mut transform) in enemies.iter_mut() {
        let curr_world_pos = Vec2::from((transform.translation.x, transform.translation.y));
        let curr_hex_pos = grid.layout.world_pos_to_hex(curr_world_pos);
        if path_tiles
            .get(*grid.entities.get(&curr_hex_pos).unwrap())
            .is_err()
            && curr_hex_pos != Hex::ZERO
        {
            commands.entity(entity).despawn();
        }

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
            if path.get(moves.path_index + 1).is_some() {
                moves.lerp += SPEED * time.delta_seconds();
                if moves.lerp > 1. {
                    moves.path_index += 1;
                    moves.lerp -= 1.;
                }
                if let Some(new_hex_pos) = path.get(moves.path_index + 1) {
                    let new_world_pos = grid.layout.hex_to_world_pos(*new_hex_pos);
                    transform.translation = curr_center
                        .lerp(new_world_pos, moves.lerp)
                        .extend(transform.translation.z);
                }
            }
        }
    }
}

pub fn execute_outside_commands(
    mut command_channel: EventReader<TDCommand>,
    mut board: Query<&mut TDState>,
) {
    if command_channel.is_empty() {
        return;
    }
    for command in command_channel.iter() {
        event!(Level::INFO, "Received command: {:#?}", command);
        match command {
            TDCommand::Toggle => {
                // TODO: should toggle a tile
            }
            TDCommand::Reset => {
                event!(Level::INFO, "matched reset, restarting...");
                let mut board_state = board.single_mut();
                board_state.restart = true;
                return;
            }
        }
    }
}

pub fn event_dispatch(
    mut command_send: EventWriter<TDCommand>,
    channel: ResMut<GameCommandChannel>,
) {
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

pub fn should_generate_new_board(game_state: Query<&TDState, With<TDBoard>>) -> bool {
    let state = game_state.single();
    state.restart
}

pub fn spawn_board_and_tiles(
    mut commands: Commands,
    mut rng: ResMut<TDRng>,
    tile_visuals: Res<TileVisuals>,
    config: Res<GameConfig>,
) {
    let layout = HexLayout {
        hex_size: config.hex_size,
        ..default()
    };
    let board = commands
        .spawn((
            SpatialBundle {
                visibility: Visibility::Visible,
                ..Default::default()
            },
            TDBoard,
            TDState {
                restart: false,
                recalculate_enemy_paths: true,
            },
        ))
        .id();
    // find spawn locations
    let spawns: Vec<Hex> = DiagonalDirection::iter()
        .map(|d| {
            Hex::ZERO
                .ring_edge(config.map_radius, d)
                .choose(&mut rng.0)
                .unwrap()
        })
        .collect();
    // create tile entities
    let entities: HashMap<Hex, Entity> = Hex::ZERO
        .spiral_range(0..=config.map_radius)
        .enumerate()
        .map(|(_i, coord)| {
            let random = rng.0.gen_range(0..=3); // TODO: make this percentage based in config
            let pos = layout.hex_to_world_pos(coord);
            // tile type
            let tile_type = if coord == Hex::ZERO {
                TileType::Goal
            } else {
                match random {
                    0..=2 => TileType::Plains,
                    _ => TileType::Mountain,
                }
            };
            // material
            let material_type = if spawns.contains(&coord) {
                MaterialType::Spawn
            } else {
                tile_type.material_type()
            };
            let material = tile_visuals.materials.get(&material_type).unwrap().clone();

            // mesh
            let mesh = tile_visuals
                .meshes
                .get(&MeshType::Hex)
                .unwrap()
                .clone()
                .into();

            let mut child = commands.spawn((
                ColorMesh2dBundle {
                    mesh,
                    material,
                    transform: Transform::from_xyz(pos.x, pos.y, 1.0).with_scale(Vec3::splat(0.9)),
                    ..default()
                },
                tile_type,
                Coords(coord),
            ));
            child.set_parent(board);
            match material_type {
                MaterialType::Goal => {
                    child.insert(IsGoal);
                }
                MaterialType::Spawn => {
                    child.insert(IsSpawn);
                }
                _ => {}
            }
            (coord, child.id())
        })
        .collect();

    commands.insert_resource(HexGrid {
        entities,
        layout,
        paths: None,
        spawn_points: spawns,
        enemy_spawn_rate: Timer::from_seconds(0.2, TimerMode::Repeating),
        damaging_rate: Timer::from_seconds(0.3, TimerMode::Repeating),
    })
}

pub fn setup_resources(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    config: Res<GameConfig>,
) {
    event!(Level::INFO, "Setting up resources");

    let seed = if let Some(s) = config.seed {
        s
    } else {
        rand::random()
    };
    let rng = StdRng::seed_from_u64(seed);
    let layout = HexLayout {
        hex_size: config.hex_size,
        ..default()
    };
    // mesh & materials
    let hex_mesh = meshes.add(hexagonal_plane(&layout));
    let enemy_mesh: Handle<Mesh> = meshes.add(shape::Circle::new(5.).into());
    let tower_mesh: Handle<Mesh> = meshes.add(shape::Quad::new(Vec2::new(8., 8.)).into());

    let plains_mat = materials.add(Color::GREEN.into());
    let mountain_mat = materials.add(Color::DARK_GRAY.into());
    let path_mat = materials.add(Color::WHITE.into());
    let spawn_mat = materials.add(Color::ORANGE_RED.into());
    let target_mat = materials.add(Color::YELLOW.into());
    let goal_mat = materials.add(Color::MIDNIGHT_BLUE.into());
    let damaging_mat_low = materials.add(Color::YELLOW.into());
    let damaging_mat_medium = materials.add(Color::ORANGE.into());
    let damaging_mat_high = materials.add(Color::CRIMSON.into());

    let enemy_mat = materials.add(Color::PURPLE.into());

    let tower_mat_small = materials.add(Color::ORANGE.into());
    let tower_mat_medium = materials.add(Color::ORANGE_RED.into());
    let tower_mat_large = materials.add(Color::CRIMSON.into());

    let tile_meshes = HashMap::from([(MeshType::Hex, hex_mesh)]);
    let enemy_meshes = HashMap::from([(MeshType::Enemy, enemy_mesh)]);
    let tower_meshes = HashMap::from([(MeshType::Tower, tower_mesh)]);

    let tile_materials = HashMap::from([
        (MaterialType::Plains, plains_mat),
        (MaterialType::Mountain, mountain_mat),
        (MaterialType::Path, path_mat),
        (MaterialType::Spawn, spawn_mat),
        (MaterialType::Goal, goal_mat),
        (MaterialType::Target, target_mat),
    ]);
    let enemy_materials = HashMap::from([(MaterialType::Enemy, enemy_mat)]);
    let tower_materials = HashMap::from([
        (TowerType::Small, tower_mat_small),
        (TowerType::Medium, tower_mat_medium),
        (TowerType::Large, tower_mat_large),
    ]);
    let damaging_materials = HashMap::from([
        (DamageLevel::Low, damaging_mat_low),
        (DamageLevel::Medium, damaging_mat_medium),
        (DamageLevel::High, damaging_mat_high),
    ]);

    event!(Level::INFO, "Tile Visuals");
    commands.insert_resource(TileVisuals {
        meshes: tile_meshes,
        materials: tile_materials,
        damaging_materials,
    });
    event!(Level::INFO, "Enemy Visuals");
    commands.insert_resource(EnemyVisuals {
        meshes: enemy_meshes,
        materials: enemy_materials,
    });
    event!(Level::INFO, "Tower Visuals");
    commands.insert_resource(TowerVisuals {
        meshes: tower_meshes,
        materials: tower_materials,
    });
    event!(Level::INFO, "Rng");
    commands.insert_resource(TDRng(rng));
}

pub fn handle_new_towers(
    towers: Query<(&Tower, &Coords), Added<Tower>>,
    mut tiles: Query<Option<&mut Damaging>, With<OnPath>>,
    mut commands: Commands,
    grid: Res<HexGrid>,
    tile_visuals: Res<TileVisuals>,
) {
    for (tower, hex_pos) in towers.iter() {
        let hex_in_range: Vec<_> = hex_pos
            .0
            .spiral_range(0..=tower.tower_type.range())
            .collect();

        for hex in hex_in_range.iter() {
            if let Some(entity) = grid.entities.get(hex) {
                if let Ok(is_damaging) = tiles.get_mut(*entity) {
                    let damage = tower.tower_type.damage();
                    if let Some(mut damaging) = is_damaging {
                        damaging.value += damage;
                    } else {
                        commands.entity(*entity).insert(Damaging { value: damage });

                        let world_pos = grid.layout.hex_to_world_pos(*hex);
                        event!(
                            Level::DEBUG,
                            "Placing damaging material at hex: {:?} word_pos: {}",
                            hex,
                            world_pos
                        );
                        let _ = commands
                            .spawn((
                                DamagingBase,
                                ColorMesh2dBundle {
                                    mesh: tile_visuals
                                        .meshes
                                        .get(&MeshType::Hex)
                                        .unwrap()
                                        .clone()
                                        .into(),
                                    material: tile_visuals
                                        .damaging_materials
                                        .get(&DamageLevel::Low)
                                        .unwrap()
                                        .clone(),
                                    transform: Transform::default()
                                        .with_scale(Vec3::splat(1.2))
                                        .with_translation(Vec3::from((0., 0., -1.))),
                                    ..default()
                                },
                            ))
                            .set_parent(*entity)
                            .id();
                        // let _ = commands.spawn(DamagingBase).set_parent(*entity).id();
                    }

                    event!(
                        Level::DEBUG,
                        "Placed damaging componend on tile at {:?}",
                        hex
                    );
                }
            }
        }
    }
    // TODO: handle removing Damaging component when tower is destroyed
}

pub fn handle_enemy_damage(
    mut enemies: Query<(Entity, &mut Enemy, &Transform)>,
    tiles: Query<Option<&Damaging>>,
    mut grid: ResMut<HexGrid>,
    mut commands: Commands,
    time: Res<Time>,
) {
    if grid.damaging_rate.tick(time.delta()).just_finished() {
        for (entity, mut enemy, transform) in enemies.iter_mut() {
            let enemy_hex_pos = grid
                .layout
                .world_pos_to_hex(transform.translation.truncate());
            if let Some(tile_entity) = grid.entities.get(&enemy_hex_pos) {
                let s = tiles.get(*tile_entity);
                match s {
                    Ok(Some(dmg)) => {
                        enemy.health = enemy.health.saturating_sub(dmg.value);
                        event!(Level::DEBUG, "Enemy lost life. Health: {}", enemy.health);
                    }
                    _ => continue,
                }
            }
            if enemy.health == 0 {
                event!(Level::DEBUG, "Enemy died at {:?}", enemy_hex_pos);
                commands.entity(entity).despawn();
            }
        }
    }
}

pub fn handle_changed_paths(
    mut commands: Commands,
    mut removed: RemovedComponents<OnPath>,
    damaging_tiles: Query<&Children, &Damaging>,
) {
    // remove the damaging base
    for removed_path in removed.iter() {
        if let Ok(children) = damaging_tiles.get(removed_path) {
            for child in children.iter() {
                commands.entity(*child).despawn();
            }
            commands.entity(removed_path).remove::<Damaging>();
        }
    }
}
