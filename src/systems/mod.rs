use crate::components::*;
use crate::resources::*;
use crate::utils::*;
use crate::AppState;
use bevy::prelude::*;
use hexx::HexLayout;
use rand::{rngs::StdRng, SeedableRng};
use std::collections::HashMap;
use tracing::{event, Level};

mod communication;
mod enemies;
mod input;
mod render;
mod tiles;
mod towers;
mod ui;

pub use communication::*;
pub use enemies::*;
pub use input::*;
pub use render::*;
pub use tiles::*;
pub use towers::*;
pub use ui::*;

pub fn destroy_board(mut commands: Commands, board: Query<Entity, With<TDBoard>>) {
    match board.get_single() {
        Ok(entity) => commands.entity(entity).despawn_recursive(),
        Err(e) => {
            event!(Level::WARN, "Received error from destroy_board: {e}");
        }
    }
}

// ticks the game_over timer upon entering the GameOver state
pub fn game_over_timer(
    mut game_timer: Query<&mut TDTimers>,
    time: Res<Time>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let mut td_timers = game_timer.single_mut();
    td_timers.game_over_timer.tick(time.delta());
    if td_timers.game_over_timer.just_finished() {
        event!(Level::INFO, "Game over timer over!");
        next_state.set(AppState::Restart);
    }
}

// ticks the game timer
pub fn game_timer(
    mut game_timer: Query<&mut GameTimer>,
    time: Res<Time>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let mut game_timer = game_timer.single_mut();
    game_timer.0.tick(time.delta());
    if game_timer.0.just_finished() {
        event!(Level::INFO, "Game over!");
        next_state.set(AppState::GameOver);
    }
}

// sets up camera
pub fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), TDCamera));
}

// inserts resources (visuals and rng)
pub fn setup_resources(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    config: Res<Config>,
) {
    event!(Level::INFO, "Setting up resources");

    let seed = if let Some(s) = config.0.seed {
        s
    } else {
        rand::random()
    };
    let rng = StdRng::seed_from_u64(seed);
    let layout = HexLayout {
        hex_size: config.0.hex_size,
        ..default()
    };
    // meshes
    let hex_mesh = meshes.add(hexagonal_plane(&layout));
    let enemy_mesh: Handle<Mesh> = meshes.add(shape::Circle::new(5.).into());
    let tower_mesh: Handle<Mesh> = meshes.add(shape::Quad::new(Vec2::new(8., 8.)).into());

    // materials
    let plains_mat = materials.add(Color::GREEN.into());
    let mountain_mat = materials.add(Color::DARK_GRAY.into());
    let path_mat = materials.add(Color::WHITE.into());
    let spawn_mat = materials.add(Color::ORANGE_RED.into());
    let target_mat = materials.add(Color::YELLOW.into());
    let goal_mat = materials.add(Color::MIDNIGHT_BLUE.into());
    let damaging_mat_low = materials.add(
        (Color::YELLOW)
            .with_a(config.0.tower_config.tower_damage_alpha)
            .into(),
    );
    let damaging_mat_medium = materials.add(
        (Color::ORANGE)
            .with_a(config.0.tower_config.tower_damage_alpha)
            .into(),
    );
    let damaging_mat_high = materials.add(
        (Color::CRIMSON)
            .with_a(config.0.tower_config.tower_damage_alpha)
            .into(),
    );

    let enemy_mat = materials.add(Color::BLACK.into());

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

// spawns the "game over" text upon game over timer resolution
pub fn show_game_over_text(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    board: Query<(Entity, &ScoreBoard), With<TDBoard>>,
) {
    let (board, score_board) = board.single();
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let text_style = TextStyle {
        font: font.clone(),
        font_size: 100.0,
        color: Color::BLACK,
    };
    let res = if score_board.enemy_score < score_board.player_score {
        "WIN"
    } else {
        "LOSE"
    };
    let text = format!("Game over!\nYOU {}", res);
    let text_alignment = TextAlignment::Center;
    commands
        .spawn((Text2dBundle {
            text: Text::from_section(text, text_style).with_alignment(text_alignment),
            transform: Transform::default().with_translation(Vec3 {
                z: 10.,
                ..default()
            }),
            ..default()
        },))
        .set_parent(board);
}
