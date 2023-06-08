use crate::communication::*;
use crate::components::*;
use crate::config::TowerConfig;
use crate::resources::*;
use crate::utils::*;
use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use tracing::event;
use tracing::Level;

pub fn handle_new_towers(
    towers: Query<(Entity, &Tower, &Coords), Or<(Changed<Tower>, Changed<Refresh>)>>,
    mut tiles: Query<Option<&mut Damaging>, With<OnPath>>,
    mut commands: Commands,
    grid: Query<&HexGrid>,
) {
    let grid = grid.single();
    for (entity, tower, hex_pos) in towers.iter() {
        for hex in hex_pos.0.spiral_range(0..=tower.tower_type.range()) {
            if let Some(entity) = grid.entities.get(&hex) {
                if let Ok(is_damaging) = tiles.get_mut(*entity) {
                    let damage = tower.tower_type.damage();
                    if let Some(mut damaging) = is_damaging {
                        event!(
                            Level::INFO,
                            "Added damage to damaging tile at hex: {:?}",
                            hex_pos
                        );
                        damaging.value += damage;
                    } else {
                        event!(Level::INFO, "Inserted damaging at hex {:?}", hex);
                        commands.entity(*entity).insert(Damaging { value: damage });
                    }
                }
            }
        }
        if let Some(mut entity_commands) = commands.get_entity(entity) {
            entity_commands.remove::<Refresh>();
        }
    }
}

pub fn refresh_damaging_tiles(
    mut commands: Commands,
    mut redraw_tower_damage: EventReader<RefreshTowerDamage>,
    towers: Query<Entity, With<Tower>>,
) {
    if redraw_tower_damage.is_empty() {
        return;
    }
    for entity in towers.iter() {
        event!(Level::INFO, "Attempting to refresh tower damage");
        if let Some(mut entity_commands) = commands.get_entity(entity) {
            entity_commands.insert(Refresh);
        }
    }
    redraw_tower_damage.clear();
}

// removes towers on path
pub fn remove_towers_on_path(
    mut commands: Commands,
    tiles_on_path_with_tower: Query<(Entity, &Children), (With<HasTower>, With<OnPath>)>,
    towers: Query<Entity, With<Tower>>,
) {
    for (tile_entity, children) in tiles_on_path_with_tower.iter() {
        for child in children.iter() {
            if let Ok(tower_entity) = towers.get(*child) {
                commands.entity(tower_entity).despawn();
                commands.entity(tile_entity).remove::<HasTower>();
                // TODO: remove damage from existing damage bases here?
            }
        }
    }
}

// spawns towers from event channel
#[allow(clippy::type_complexity)]
pub fn spawn_tower(
    mut commands: Commands,
    mut create_tower: EventReader<CreateTower>,
    tower_visuals: Res<TowerVisuals>,
    mut budget: Query<&mut Budget>,
    grid: Query<&HexGrid>,
    config: Res<Config>,
    unplaceable_tiles: Query<(
        Option<&HasTower>,
        Option<&OnPath>,
        Option<&IsGoal>,
        Option<&IsSpawn>,
    )>,
) {
    if create_tower.is_empty() {
        return;
    }
    let grid = grid.single();
    for t in create_tower.iter() {
        let tile_entity = *grid.entities.get(&t.hex_pos).unwrap();
        // if there is already a tower on the tile, skip it
        let (has_tower, on_path, is_goal, is_spawn) = unplaceable_tiles.get(tile_entity).unwrap();
        if has_tower.is_some() || on_path.is_some() || is_goal.is_some() || is_spawn.is_some() {
            continue;
        }

        let tower_type = t.tower_type;
        let (cost, scale) = {
            let TowerConfig { cost, scale, .. } =
                config.0.tower_config.tower_type.get(&t.tower_type).unwrap();
            (*cost, *scale)
        };
        let mut budget = budget.single_mut();
        if cost > budget.0 {
            event!(
                Level::WARN,
                "Not enough resources for tower. Budget: {}, Cost: {}",
                budget.0,
                cost
            );
            continue;
        } else {
            budget.0 = budget.0.saturating_sub(cost);
            commands
                .spawn((
                    MaterialMesh2dBundle {
                        mesh: tower_visuals
                            .meshes
                            .get(&MeshType::Tower)
                            .unwrap()
                            .clone()
                            .into(),
                        material: tower_visuals.materials.get(&tower_type).unwrap().clone(),
                        transform: Transform::default()
                            .with_translation(Vec3 { z: 2., ..default() })
                            .with_scale(Vec3::splat(scale)),
                        ..default()
                    },
                    Tower { tower_type, cost },
                    Coords(t.hex_pos),
                ))
                .set_parent(tile_entity);
            commands.entity(tile_entity).insert(HasTower);
        }
    }
}
