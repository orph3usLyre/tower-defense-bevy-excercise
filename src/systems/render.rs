use crate::{components::*, resources::TileVisuals, utils::*};
use bevy::prelude::*;
use tracing::{event, Level};

// Renders all tiles
pub fn render_tiles(
    mut commands: Commands,
    tile_visuals: Res<TileVisuals>,
    tiles: Query<
        (
            Entity,
            &Tile,
            &Coords,
            Option<&Transform>,
            Option<&IsSpawn>,
            Option<&IsGoal>,
            Option<&OnPath>,
        ),
        Or<(Changed<Tile>, Changed<OnPath>, Changed<Refresh>)>,
    >,
    grid: Query<&HexGrid>,
) {
    let grid = grid.single();
    for (entity, tile, coords, has_transform, is_spawn, is_goal, on_path) in tiles.iter() {
        let material_type = match (
            tile.is_cursor,
            is_goal.is_some(),
            is_spawn.is_some(),
            on_path.is_some(),
        ) {
            (true, _, _, _) => MaterialType::Target,
            (_, true, _, _) => MaterialType::Goal,
            (_, _, true, _) => MaterialType::Spawn,
            (_, _, _, true) => MaterialType::Path,
            _ => tile.tile_type.material_type(),
        };
        let material = tile_visuals.materials.get(&material_type).unwrap().clone();
        // TODO: check if `none` then always material
        if has_transform.is_some() {
            // then tile just needs a material change
            if let Some(mut entity_commands) = commands.get_entity(entity) {
                entity_commands.insert(material);

                event!(
                    Level::DEBUG,
                    "Rendered {:?} as {:?}",
                    coords.0,
                    material_type
                );
            }
            // Tile hasn't been given the 2d bundle
        } else {
            let mesh = tile_visuals
                .meshes
                .get(&MeshType::Hex)
                .unwrap()
                .clone()
                .into();
            let pos = grid.layout.hex_to_world_pos(coords.0);
            if let Some(mut entity_commands) = commands.get_entity(entity) {
                entity_commands.insert(ColorMesh2dBundle {
                    mesh,
                    material,
                    transform: Transform::from_xyz(pos.x, pos.y, 1.0).with_scale(Vec3::splat(0.9)),
                    ..default()
                });
            }
        }
        if let Some(mut entity_commands) = commands.get_entity(entity) {
            entity_commands.remove::<Refresh>();
        }
    }
}

// renders damage aoe
// TODO: Move this into `render_tiles` and simplify
pub fn render_tower_aoe(
    mut commands: Commands,
    tiles: Query<
        (Entity, &Coords, &Damaging, Option<&Children>),
        (With<OnPath>, Changed<Damaging>),
    >,
    damaging_base: Query<Entity, With<DamagingBase>>,
    tile_visuals: Res<TileVisuals>,
) {
    for (entity, coords, damaging, children) in tiles.iter() {
        event!(Level::DEBUG, "Render tower aoe received hex {:?}", coords.0);
        if children.is_none() || children.is_some_and(|c| c.is_empty()) {
            event!(
                Level::DEBUG,
                "Inserting visual component for childless damaging tile at hex: {:?}",
                coords.0
            );
            commands
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
                            .get(&DamageLevel::get_level(damaging.value))
                            .unwrap()
                            .clone(),
                        transform: Transform::default().with_translation(Vec3::from((0., 0., 2.))),
                        ..default()
                    },
                ))
                .set_parent(entity);
        }
        if let Some(children) = children {
            for child in children.iter() {
                if let Ok(child) = damaging_base.get(*child) {
                    event!(
                        Level::DEBUG,
                        "Updating visual on existing aoe at hex {:?}",
                        coords.0
                    );
                    commands.entity(child).insert(
                        tile_visuals
                            .damaging_materials
                            .get(&DamageLevel::get_level(damaging.value))
                            .unwrap()
                            .clone(),
                    );
                }
            }
        }
    }
}
