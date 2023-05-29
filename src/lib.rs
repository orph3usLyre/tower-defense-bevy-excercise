use bevy::prelude::*;
use bevy_egui::EguiPlugin;
pub use communication::{parse_command, TDCommand};
use crossbeam_channel::{unbounded, Sender};

mod communication;
mod components;
mod events;
mod resources;
mod systems;
mod utils;

use resources::{GameCommandChannel, ScoreBoard};
use systems::*;

/// World size of the hexagons (outer radius)
pub const HEX_SIZE: Vec2 = Vec2::splat(10.0);
pub const MAP_RADIUS: u32 = 20;
pub const BUDGET: u32 = 13;

pub fn setup_tower_defense() -> (App, Sender<TDCommand>) {
    let mut app = App::new();
    let (tx, rx) = unbounded::<TDCommand>();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            resolution: (1_500.0, 1_500.0).into(),
            fit_canvas_to_parent: true,
            resizable: true,
            ..default()
        }),
        ..default()
    }))
    .insert_resource(ClearColor(Color::rgb(0.1, 0.0, 0.0)))
    .insert_resource(ScoreBoard::default())
    .insert_resource(GameCommandChannel(rx))
    .add_event::<TDCommand>()
    .add_plugin(EguiPlugin)
    // Systems that create Egui widgets should be run during the `CoreSet::Update` set,
    // or after the `EguiSet::BeginFrame` system (which belongs to the `CoreSet::PreUpdate` set).
    .add_startup_system(setup_camera)
    .add_startup_system(setup_grid)
    .add_system(show_ui)
    .add_system(recalculate_enemy_path)
    .add_system(handle_input.before(handle_color_change))
    .add_system(handle_color_change)
    .add_system(spawn_enemies)
    .add_system(handle_enemy_movement)
    .add_system(execute_outside_commands)
    .add_system(restart)
    .add_system(event_dispatch.before(execute_outside_commands));
    (app, tx)
}
