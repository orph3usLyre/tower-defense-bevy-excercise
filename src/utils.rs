use std::collections::HashMap;

use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use hexx::{DiagonalDirection, Hex, HexLayout, PlaneMeshBuilder};
use rand::prelude::*;

use crate::{components::*, resources::HexGrid, HEX_SIZE, MAP_RADIUS};

// Compute a bevy mesh from the layout
pub fn hexagonal_plane(hex_layout: &HexLayout) -> Mesh {
    let mesh_info = PlaneMeshBuilder::new(hex_layout).facing(Vec3::Z).build();
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh_info.vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_info.normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, mesh_info.uvs);
    mesh.set_indices(Some(Indices::U16(mesh_info.indices)));
    mesh
}

pub fn generate_board(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut rng = rand::thread_rng();
    let layout = HexLayout {
        hex_size: HEX_SIZE,
        ..default()
    };
    // mesh & materials
    let hex_mesh = meshes.add(hexagonal_plane(&layout));
    let enemy_mesh = meshes.add(shape::Circle::new(5.).into()).into();

    let plains_mat = materials.add(Color::GREEN.into());
    let mountain_mat = materials.add(Color::DARK_GRAY.into());
    let path_mat = materials.add(Color::WHITE.into());
    let spawn_mat = materials.add(Color::ORANGE_RED.into());
    let target_mat = materials.add(Color::YELLOW.into());
    let goal_mat = materials.add(Color::MIDNIGHT_BLUE.into());
    let enemy_mat = materials.add(Color::PURPLE.into());

    // generate board entity
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
        .map(|d| Hex::ZERO.ring_edge(MAP_RADIUS, d).choose(&mut rng).unwrap())
        .collect();

    // setup entities
    let entities: HashMap<Hex, Entity> = Hex::ZERO
        .spiral_range(0..=MAP_RADIUS)
        .enumerate()
        .map(|(_i, coord)| {
            let cost = rng.gen_range(0..=3);
            let pos = layout.hex_to_world_pos(coord);
            let (material, tile_type, spawn, goal, _tile_material_type) = if coord == Hex::ZERO {
                (
                    goal_mat.clone(),
                    TileType::Goal,
                    false,
                    true,
                    MaterialType::Goal,
                )
            } else if spawns.contains(&coord) {
                (
                    spawn_mat.clone(),
                    TileType::Plains,
                    true,
                    false,
                    MaterialType::Spawn,
                )
            } else {
                match cost {
                    0..=2 => (
                        plains_mat.clone(),
                        TileType::Plains,
                        false,
                        false,
                        MaterialType::Plains,
                    ),
                    3 => (
                        mountain_mat.clone(),
                        TileType::Mountain,
                        false,
                        false,
                        MaterialType::Mountain,
                    ),
                    _ => unreachable!(),
                }
            };

            let mut child = commands.spawn((
                ColorMesh2dBundle {
                    mesh: hex_mesh.clone().into(),
                    material,
                    transform: Transform::from_xyz(pos.x, pos.y, 0.0).with_scale(Vec3::splat(0.9)),
                    ..default()
                },
                // HexTile::new(tile_type),
                tile_type,
            ));
            child.set_parent(board);
            if spawn {
                child.insert(IsSpawn);
            }
            if goal {
                child.insert(IsGoal);
            }

            let child = child.id();

            (coord, child)
        })
        .collect();

    // let paths: HashMap<usize, Vec<Hex>> = spawns
    //     .iter()
    //     .enumerate()
    //     .map(|(i, spawn)| {
    //         let path: Vec<Hex> = a_star(*spawn, Hex::ZERO, |hex| {
    //             if entities.contains_key(&hex) {
    //                 let entity = entities.get(&hex).unwrap();
    //                 match .get(*entity).unwrap() {
    //                     TileType::Plains => Some(0),
    //                     TileType::Mountain => Some(1000),
    //                     TileType::Goal => Some(0),
    //                 }
    //             } else {
    //                 None
    //             }
    //         })
    //         .unwrap();
    //         path.into_iter().for_each(|hex| {
    //             if hex != Hex::ZERO {
    //                 let entity = entities.get(&hex).copied().unwrap();
    //                 let _ = commands.entity(entity).insert(OnPath).id();
    //             }
    //         });
    //         (i, path)
    //     })
    //     .collect();

    let materials = HashMap::from([
        (MaterialType::Plains, plains_mat),
        (MaterialType::Mountain, mountain_mat),
        (MaterialType::Path, path_mat),
        (MaterialType::Spawn, spawn_mat),
        (MaterialType::Goal, goal_mat),
        (MaterialType::Target, target_mat),
        (MaterialType::Enemy, enemy_mat),
    ]);

    let meshes = HashMap::from([(MeshType::Hex, hex_mesh), (MeshType::Enemy, enemy_mesh)]);
    commands.insert_resource(HexGrid {
        entities,
        layout,
        materials,
        paths: None,
        spawn_points: spawns,
        enemy_spawn_rate: Timer::from_seconds(0.2, TimerMode::Repeating),
        meshes,
    })
}

pub fn destroy_board(commands: &mut Commands, board_entity: Entity) {
    commands.entity(board_entity).despawn_recursive();
}
