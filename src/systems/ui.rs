use crate::communication::*;
use crate::components::*;
use crate::resources::*;
use crate::AppState;
use crate::CONFIG_PATH;
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_egui::egui::Align2;
use bevy_egui::egui::ProgressBar;
use bevy_egui::{egui, EguiContexts};
use tracing::event;
use tracing::Level;

pub fn show_ui(
    mut contexts: EguiContexts,
    tiles: Query<(
        &Tile,
        &Coords,
        Option<&Damaging>,
        Option<&OnPath>,
        Option<&Children>,
    )>,
    damaging_base: Query<&DamagingBase>,
    mut config: ResMut<Config>,
    mut board_q: Query<(&mut Budget, &mut ScoreBoard, &GameTimer)>,
    mut selected_tower: ResMut<SelectedTower>,
    mut restart_channel: EventWriter<Restart>,
    mut exit: EventWriter<AppExit>,
    mut next_state: ResMut<NextState<AppState>>,
    state: Res<State<AppState>>,
) {
    let (budget, score_board, game_timer) = board_q.single_mut();
    egui::Window::new("Tower Defense")
        .anchor(Align2::LEFT_TOP, [5.0, 5.0])
        .show(contexts.ctx_mut(), |ui| {
            ui.heading("Debug");
            ui.label("Selected tile");
            if let Some((_, hex, damaging, on_path, children)) =
                tiles.iter().find(|(t, _, _, _, _)| t.is_cursor)
            {
                ui.label(format!("Coord: x: {}, y: {}", hex.0.x(), hex.0.y()));

                let text = if let Some(dmg) = damaging {
                    dmg.value.to_string()
                } else {
                    "None".to_string()
                };
                ui.label(format!("Damaging: {}", text));
                ui.label(format!(
                    "OnPath: {}",
                    if on_path.is_some() { "Yes" } else { "No" }
                ));
                let d_base = if children
                    .is_some_and(|c| c.first().is_some_and(|c| damaging_base.get(*c).is_ok()))
                {
                    "yes"
                } else {
                    "no"
                };
                ui.label(format!("Damaging base: {}", d_base));
            } else {
                ui.label("None selected".to_string());
            }

            egui::CollapsingHeader::new("Board config").show(ui, |ui| {
                ui.label("Game length");
                ui.add(egui::Slider::new(&mut config.0.game_length, 30_f32..=300.));
                ui.label("Map radius");
                ui.add(egui::Slider::new(&mut config.0.map_radius, 10..=120));
                if ui.button("+ Hex Size").clicked() {
                    config.0.hex_size += 1.;
                };
                if ui.button("- Hex Size").clicked() {
                    config.0.hex_size -= 1.;
                };
                ui.label("Starting budget");
                ui.add(egui::Slider::new(&mut config.0.starting_budget, 10..=120));
            });

            egui::CollapsingHeader::new("Enemy Config").show(ui, |ui| {
                ui.label("Enemy speed");
                ui.add(
                    egui::DragValue::new(&mut config.0.enemy_config.base_speed)
                        .speed(0.1)
                        .clamp_range(1_f32..=20.),
                );
                ui.label("Enemy spawn rate");
                ui.add(
                    egui::DragValue::new(&mut config.0.enemy_config.enemy_spawn_rate)
                        .speed(0.1)
                        .clamp_range(0.01..=20.),
                );
            });

            egui::CollapsingHeader::new("Tower Config").show(ui, |ui| {
                ui.label("Tower damage rate");
                ui.add(
                    egui::DragValue::new(&mut config.0.tower_config.damaging_rate)
                        .speed(0.1)
                        .clamp_range(0.01..=20.),
                );
                ui.label("Tower cost");
                for (t_type, config) in config.0.tower_config.tower_type.iter_mut() {
                    ui.label(format!("{:?}", t_type));
                    ui.add(
                        egui::DragValue::new(&mut config.cost)
                            .speed(0.1)
                            .clamp_range(1..=100),
                    );
                }
            });
            ui.label(format!("Current budget: {}", budget.0));
            ui.label("Score Board");
            ui.horizontal(|ui| {
                ui.label(format!("Player: {}", score_board.player_score));
                ui.label(format!("Enemy: {}", score_board.enemy_score));
            });
            ui.horizontal(|ui| {
                if ui.button("Export config").clicked() {
                    let game_config = config.0.clone();
                    match game_config.export(CONFIG_PATH) {
                        Err(e) => event!(Level::WARN, "{e}"),
                        Ok(()) => event!(Level::INFO, "Config written to {CONFIG_PATH}"),
                    }
                }
            });
            ui.horizontal(|ui| {
                if ui.button("Pause").clicked() {
                    if state.0 == AppState::Pause {
                        next_state.set(AppState::InGame);
                    } else {
                        next_state.set(AppState::Pause);
                    }
                }
            });
            ui.horizontal(|ui| {
                if ui.button("Simulate Game over").clicked() {
                    next_state.set(AppState::GameOver);
                }
            });
            ui.horizontal(|ui| {
                if ui.button("Restart Board").clicked() {
                    restart_channel.send(Restart);
                }
            });
            ui.horizontal(|ui| {
                if ui.button("Exit").clicked() {
                    exit.send(AppExit);
                }
            });
        });

    egui::Window::new("Towers")
        .anchor(Align2::RIGHT_TOP, [5.0, 5.0])
        .resizable(false)
        .show(contexts.ctx_mut(), |ui| {
            ui.label("Selected tower");
            egui::ComboBox::from_label("")
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

    egui::Window::new("Time till death")
        .constrain(true)
        .anchor(Align2::CENTER_BOTTOM, [1.0, 1.0])
        .interactable(false)
        .collapsible(false)
        .resizable(false)
        .show(contexts.ctx_mut(), |ui| {
            ui.add(ProgressBar::new(game_timer.0.percent()).show_percentage());
        });
}
