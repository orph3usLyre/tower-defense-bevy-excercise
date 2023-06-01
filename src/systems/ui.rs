use crate::communication::Restart;
use crate::components::*;
use crate::resources::*;
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

pub fn show_ui(
    mut contexts: EguiContexts,
    tiles: Query<(&Tile, &Coords, Option<&Damaging>)>,
    mut config: ResMut<Config>,
    budget: Query<&mut Budget>,
    score_board: Res<ScoreBoard>,
    mut selected_tower: ResMut<SelectedTower>,
    mut restart_channel: EventWriter<Restart>,
    mut exit: EventWriter<AppExit>,
) {
    egui::Window::new("Tower Defense").show(contexts.ctx_mut(), |ui| {
        egui::CollapsingHeader::new("Debug").show(ui, |ui| {
            ui.label("Selected tile");
            if let Some((_, hex, damaging)) = tiles.iter().find(|(t, _, _)| t.is_cursor) {
                ui.label(format!("Coord: x: {}, y: {}", hex.0.x(), hex.0.y()));

                let text = if let Some(dmg) = damaging {
                    dmg.value.to_string()
                } else {
                    "None".to_string()
                };
                ui.label(format!("Damaging: {}", text));
            } else {
                ui.label("None selected".to_string());
            }
        });

        // ui.heading("Debug");
        // ui.label("Selected tile");
        // if let Some((_, hex, damaging)) = tiles.iter().find(|(t, _, _)| t.is_cursor) {
        //     ui.label(format!("Coord: x: {}, y: {}", hex.0.x(), hex.0.y()));
        //
        //     let text = if let Some(dmg) = damaging {
        //         dmg.value.to_string()
        //     } else {
        //         "None".to_string()
        //     };
        //     ui.label(format!("Damaging: {}", text));
        // } else {
        //     ui.label("None selected".to_string());
        // }
        egui::CollapsingHeader::new("Board config").show(ui, |ui| {
            ui.label("Starting budget");
            ui.add(egui::Slider::new(&mut config.0.starting_budget, 10..=120));
            ui.label(format!("Current budget: {}", budget.single().0));
            ui.label("Map radius");
            ui.add(egui::Slider::new(&mut config.0.map_radius, 10..=120));
            if ui.button("+ Hex Size").clicked() {
                config.0.hex_size += 1.;
            };
            if ui.button("- Hex Size").clicked() {
                config.0.hex_size -= 1.;
            };
        });
        // ui.heading("Config");
        // ui.label("Starting budget");
        // ui.add(egui::Slider::new(&mut config.0.starting_budget, 10..=120));
        // ui.label(format!("Current budget: {}", budget.single().0));
        // ui.label("Map radius");
        // ui.add(egui::Slider::new(&mut config.0.map_radius, 10..=120));
        // if ui.button("+ Hex Size").clicked() {
        //     config.0.hex_size += 1.;
        // };
        // if ui.button("- Hex Size").clicked() {
        //     config.0.hex_size -= 1.;
        // };
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
        });
        ui.horizontal(|ui| {
            ui.label("Score Board");
            ui.label(format!("Score: {}", score_board.score));
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

    egui::Window::new("Towers").show(contexts.ctx_mut(), |ui| {
        ui.label("Selected tower");
        egui::ComboBox::from_label("Select tower")
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
