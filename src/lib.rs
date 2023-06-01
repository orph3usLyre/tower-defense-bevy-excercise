use bevy::prelude::*;
use bevy_egui::EguiPlugin;
pub use communication::{parse_command, TDCommand};
use config::GameConfig;
use crossbeam_channel::{unbounded, Sender};

mod communication;
mod components;
mod config;
mod resources;
mod systems;
mod utils;

use communication::*;
use resources::*;
use systems::*;

#[derive(States, PartialEq, Eq, Debug, Clone, Hash, Default)]
pub enum AppState {
    #[default]
    Setup,
    InGame,
    Restart,
}

pub fn setup_tower_defense() -> (App, Sender<TDCommand>) {
    let config = GameConfig::load("config/config.yaml");

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
    .add_state::<AppState>()
    .insert_resource(ClearColor(Color::rgb(0.1, 0.0, 0.0)))
    .insert_resource(ScoreBoard::default())
    .insert_resource(GameCommandChannel(rx))
    .insert_resource(SelectedTower::default())
    .insert_resource(Config(config))
    .add_event::<TDCommand>()
    .add_event::<CreateTower>()
    .add_event::<RecalculateEnemyPaths>()
    .add_event::<Restart>()
    .add_event::<ToggleTile>()
    .add_plugin(EguiPlugin)
    // Systems that create Egui widgets should be run during the `CoreSet::Update` set,
    // or after the `EguiSet::BeginFrame` system (which belongs to the `CoreSet::PreUpdate` set).
    .add_systems(
        (
            setup_camera,
            setup_resources,
            apply_system_buffers,
            spawn_board_and_tiles,
        )
            .chain()
            .in_schedule(OnEnter(AppState::Setup)),
    )
    .add_systems(
        (recalculate_enemy_path,)
            .chain()
            .in_schedule(OnEnter(AppState::InGame)),
    )
    .add_systems(
        (
            show_ui,
            handle_input,
            spawn_enemies,
            execute_outside_commands,
            handle_new_towers,
            spawn_tower,
            handle_enemy_damage,
            handle_removed_paths,
            handle_enemy_movement,
            event_dispatch,
            receive_restart_command,
            recalculate_enemy_path,
            toggle_tile,
            render_tower_aoe,
            camera_zoom,
        )
            .in_set(OnUpdate(AppState::InGame)),
    )
    .add_systems((remove_towers_on_path, render_tiles).in_set(OnUpdate(AppState::InGame)))
    .add_systems(
        (destroy_board, spawn_board_and_tiles)
            .chain()
            .in_schedule(OnEnter(AppState::Restart)),
    );
    (app, tx)
}
