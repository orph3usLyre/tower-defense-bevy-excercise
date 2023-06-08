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

pub const CONFIG_PATH: &str = "config/config.yaml";

// The app states that exist for the game
// The `Setup` state is equivallent to the startup systems
// The `InGame` state is the running state of the game
// The `Restart` state is responsible for resenting the game entities
// The `GameOver` state is a freeze state to show player victory/loss
#[derive(States, PartialEq, Eq, Debug, Clone, Hash, Default)]
pub enum AppState {
    #[default]
    Setup,
    InGame,
    Pause,
    Restart,
    GameOver,
}

pub fn setup_tower_defense() -> (App, Sender<TDCommand>) {
    // load game config
    let config = GameConfig::load(CONFIG_PATH);

    // setup app
    let mut app = App::new();

    // setup app channels to communicate from
    // outside of the bevy engine
    let (tx, rx) = unbounded::<TDCommand>();
    // Plugins
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            resolution: (1_500.0, 1_500.0).into(),
            fit_canvas_to_parent: true,
            resizable: true,
            ..default()
        }),
        ..default()
    }))
    .add_plugin(EguiPlugin)
    // State
    .add_state::<AppState>()
    // Resources
    .insert_resource(ClearColor(Color::rgb(0.1, 0.0, 0.0)))
    .insert_resource(GameCommandChannel(rx))
    .insert_resource(SelectedTower::default())
    .insert_resource(Config(config))
    // Events
    .add_event::<TDCommand>()
    .add_event::<CreateTower>()
    .add_event::<RecalculateEnemyPaths>()
    .add_event::<Restart>()
    .add_event::<GameOver>()
    .add_event::<ToggleTile>()
    .add_event::<RefreshTowerDamage>()
    // Systems
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
    .add_system(recalculate_enemy_path.in_schedule(OnEnter(AppState::InGame)))
    .add_systems(
        (
            camera_zoom,
            event_dispatch,
            execute_outside_commands,
            handle_input,
            handle_enemy_damage,
            handle_enemy_movement,
            handle_new_towers,
            handle_removed_paths,
            show_ui,
            spawn_enemies,
            spawn_tower,
            recalculate_enemy_path,
            receive_restart_command,
            render_tiles,
            render_tower_aoe,
        )
            .in_set(OnUpdate(AppState::InGame)),
    )
    .add_systems(
        (
            game_timer,
            toggle_tile,
            refresh_damaging_tiles,
            remove_towers_on_path,
        )
            .in_set(OnUpdate(AppState::InGame)),
    )
    .add_systems((show_ui, handle_input, camera_zoom).in_set(OnUpdate(AppState::Pause)))
    .add_systems(
        (destroy_board, spawn_board_and_tiles)
            .chain()
            .in_schedule(OnEnter(AppState::Restart)),
    )
    .add_system(show_game_over_text.in_schedule(OnEnter(AppState::GameOver)))
    .add_system(game_over_timer.in_set(OnUpdate(AppState::GameOver)));

    (app, tx)
}
