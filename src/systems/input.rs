use crate::communication::*;
use crate::components::*;
use crate::resources::*;
use bevy::input::mouse::MouseWheel;
use bevy::{prelude::*, window::PrimaryWindow};
use hexx::Hex;
use tracing::{event, Level};

// Input interaction
#[allow(clippy::too_many_arguments)]
pub fn handle_input(
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<TDCamera>>,
    mut tiles: Query<(Option<&OnPath>, Option<&IsGoal>, Option<&HasTower>)>,
    mut cursor: Query<&mut Tile>,
    mut current: Local<Hex>,
    grid: ResMut<HexGrid>,
    buttons: Res<Input<MouseButton>>,
    selected_tower: Res<SelectedTower>,
    mut tower_create: EventWriter<CreateTower>,
    mut recalculate_paths: EventWriter<RecalculateEnemyPaths>,
    mut toggle_tiles: EventWriter<ToggleTile>,
) {
    let window = windows.single();
    let (camera, camera_transform) = camera_q.single();
    // get normalized cursor position based on camera viewport
    if let Some(pos) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        let hex_pos = grid.layout.world_pos_to_hex(pos);

        // change tiles according to buttons
        if buttons.just_pressed(MouseButton::Left) {
            event!(Level::INFO, "Left click pressed at {:?}", hex_pos);
            if let Some(entity) = grid.entities.get(&hex_pos) {
                // functionality for left button
                let (on_path, _is_goal, _) = tiles.get_mut(*entity).unwrap();
                if on_path.is_some() {
                    event!(Level::INFO, "Clicked tile on path");
                    // send recaulculate command
                    recalculate_paths.send(RecalculateEnemyPaths);
                }
                // send toggle tile command
                toggle_tiles.send(ToggleTile { hex_pos })
            }
        }
        // functionality for right button press
        if buttons.just_pressed(MouseButton::Right) {
            if let Some(entity) = grid.entities.get(&hex_pos) {
                if let Ok((on_path, is_goal, has_tower)) = tiles.get(*entity) {
                    event!(Level::INFO, "Right clicked to place tower");
                    if is_goal.is_some() || on_path.is_some() || has_tower.is_some() {
                        return;
                    } else {
                        // spawn tower command
                        tower_create.send(CreateTower {
                            tower_type: selected_tower.selected,
                            hex_pos,
                        });
                    }
                }
            }
        }

        if hex_pos == *current {
            return;
        }
        *current = hex_pos;

        // remove old cursors
        // TODO: find a way to make this better
        cursor
            .iter_mut()
            .filter(|t| t.is_cursor)
            .for_each(|mut t| t.is_cursor = false);
        // add cursor marker
        if let Some(cursor_entity) = grid.entities.get(&hex_pos) {
            if let Ok(mut tile) = cursor.get_mut(*cursor_entity) {
                tile.is_cursor = true;
            }
        }
    }
}

pub fn camera_zoom(
    mut camera_projection: Query<&mut OrthographicProjection, With<TDCamera>>,
    time: Res<Time>,
    mut wheel: EventReader<MouseWheel>,
    config: Res<Config>,
) {
    let delta_zoom: f32 = wheel.iter().map(|e| e.y).sum();
    if delta_zoom == 0. {
        return;
    }
    let mut projection = camera_projection.single_mut();
    let mut log_scale = projection.scale.ln();
    // log_scale -= 0.1 * time.delta_seconds() * config.0.zoom_speed * delta_zoom;
    log_scale -= time.delta_seconds() * config.0.zoom_speed * delta_zoom;
    log_scale = log_scale.exp();
    projection.scale = log_scale.clamp(-50., 50.);

    event!(Level::DEBUG, "Current zoom scale: {}", projection.scale);
}
