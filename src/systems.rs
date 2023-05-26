use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::{egui, EguiContexts};
use hexx::{algorithms::*, Hex, HexLayout};
use rand::prelude::*;
use std::collections::HashSet;

use crate::resources::HexGrid;
use crate::utils::*;

// Systems
pub fn ui_example_system(mut contexts: EguiContexts) {
    egui::Window::new("Hello").show(contexts.ctx_mut(), |ui| {
        ui.label("world");
    });
}

pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

pub fn setup_grid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let layout = HexLayout {
        hex_size: crate::HEX_SIZE,
        ..default()
    };
    let mesh = meshes.add(hexagonal_plane(&layout));
    let plains_mat = materials.add(Color::WHITE.into());
    let forest_mat = materials.add(Color::GREEN.into());
    let desert_mat = materials.add(Color::YELLOW.into());
    let wall_mat = materials.add(Color::DARK_GRAY.into());

    let mut rng = rand::thread_rng();

    let entities = Hex::ZERO
        .spiral_range(0..=crate::MAP_RADIUS)
        .enumerate()
        .map(|(_i, coord)| {
            let cost = rng.gen_range(0..=3);
            let pos = layout.hex_to_world_pos(coord);
            let material = match cost {
                0 => plains_mat.clone(),
                1 => forest_mat.clone(),
                2 => desert_mat.clone(),
                3 => wall_mat.clone(),
                _ => unreachable!(),
            };
            let cost = if (0..3).contains(&cost) {
                Some(cost)
            } else {
                None
            };
            let entity = commands
                .spawn(ColorMesh2dBundle {
                    mesh: mesh.clone().into(),
                    material,
                    transform: Transform::from_xyz(pos.x, pos.y, 0.0).with_scale(Vec3::splat(1.)),
                    ..default()
                })
                .id();
            (coord, (cost, entity))
        })
        .collect();
    commands.insert_resource(HexGrid {
        entities,
        reachable_entities: Default::default(),
        layout,
    })
}

// Input interaction
pub fn handle_input(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut tile_transforms: Query<(Entity, &mut Transform)>,
    mut current: Local<Hex>,
    mut grid: ResMut<HexGrid>,
) {
    let window = windows.single();
    if let Some(pos) = window.cursor_position() {
        let pos = pos - Vec2::new(window.width(), window.height()) / 2.0;
        let hex_pos = grid.layout.world_pos_to_hex(pos);

        if hex_pos == *current {
            return;
        }
        *current = hex_pos;

        let field_of_movement = field_of_movement(hex_pos, crate::BUDGET, |h| {
            grid.entities.get(&h).and_then(|c| c.0)
        });

        let reachable_entities: HashSet<_> = field_of_movement
            .into_iter()
            .filter_map(|h| grid.entities.get(&h).map(|&(_, ent)| ent))
            .collect();
        for (entity, mut transform) in tile_transforms.iter_mut() {
            if reachable_entities.contains(&entity) {
                *transform = transform.with_scale(Vec3::splat(0.9));
            } else {
                *transform = transform.with_scale(Vec3::splat(1.));
            }
        }

        grid.reachable_entities = reachable_entities;
    }
}

// handle terminal commands
