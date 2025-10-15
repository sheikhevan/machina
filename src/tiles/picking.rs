use bevy::picking::events::{Click, Out, Over, Pointer};
use bevy::picking::hover::PickingInteraction;
use bevy::{
    picking::{
        PickingSystems,
        backend::{HitData, PointerHits},
        pointer::{PointerId, PointerLocation},
    },
    prelude::*,
    window::PrimaryWindow,
};
use bevy_ecs_tilemap::prelude::*;

// The code on this page was adapted from dpogorzelski on GitHub.
// Original code can be found at: https://github.com/StarArawn/bevy_ecs_tilemap/issues/572

pub struct TilemapBackend;

impl Plugin for TilemapBackend {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, tile_picking.in_set(PickingSystems::Backend))
            .add_systems(Update, highlight_hovered_tile);
    }
}

fn tile_picking(
    q_pointers: Query<(&PointerId, &PointerLocation)>,
    q_cameras: Query<(Entity, &Camera, &GlobalTransform, &Projection)>,
    q_primary_window: Query<Entity, With<PrimaryWindow>>,
    q_tilemap: Query<(
        &TilemapSize,
        &TilemapGridSize,
        &TilemapType,
        &TileStorage,
        &TilemapTileSize,
        &TilemapAnchor,
        &GlobalTransform,
        &ViewVisibility,
    )>,
    q_tile: Query<&TileVisible>,
    mut output: MessageWriter<PointerHits>,
) {
    for (p_id, p_loc) in q_pointers
        .iter()
        .filter_map(|(p_id, p_loc)| p_loc.location().map(|l| (p_id, l)))
    {
        let Some((cam_entity, camera, cam_transform, cam_ortho)) = q_cameras
            .iter()
            .filter(|(_, camera, _, _)| camera.is_active)
            .find(|(_, camera, _, _)| {
                camera
                    .target
                    .normalize(Some(match q_primary_window.single() {
                        Ok(w) => w,
                        Err(_) => return false,
                    }))
                    .unwrap()
                    == p_loc.target
            })
        else {
            continue;
        };

        let Projection::Orthographic(ortho) = cam_ortho else {
            continue;
        };

        let Ok(cursor_pos_world) = camera.viewport_to_world_2d(cam_transform, p_loc.position)
        else {
            continue;
        };

        let picks = q_tilemap
            .iter()
            .filter(|(.., vis)| vis.get())
            .filter_map(|(t_s, tgs, tty, t_store, tile_size, anchor, gt, ..)| {
                // if blocked {
                //     return None;
                // }
                let in_map_pos: Vec2 = {
                    let pos = Vec4::from((cursor_pos_world, 0., 1.));
                    let in_map_pos = gt.to_matrix().inverse() * pos;
                    in_map_pos.xy()
                };
                let picked: Entity =
                    TilePos::from_world_pos(&in_map_pos, t_s, tgs, tile_size, tty, anchor)
                        .and_then(|tile_pos| t_store.get(&tile_pos))?;
                let vis = q_tile.get(picked).ok()?;
                if !vis.0 {
                    return None;
                }
                // blocked = pck.is_some_and(|p| p.should_block_lower);
                let depth = -ortho.near - gt.translation().z;
                Some((picked, HitData::new(cam_entity, depth, None, None)))
            })
            .collect();
        // f32 required by PointerHits
        #[allow(clippy::cast_precision_loss)]
        let order = camera.order as f32;
        output.write(PointerHits::new(*p_id, picks, order));
    }
}

fn highlight_hovered_tile(
    mut q_tiles: Query<(&mut TileColor, &PickingInteraction), Changed<PickingInteraction>>,
) {
    for (mut color, interaction) in q_tiles.iter_mut() {
        match *interaction {
            PickingInteraction::Hovered => {
                *color = TileColor(Color::srgba(1.0, 1.0, 0.5, 1.0)); // Yellow when hovered
            }
            PickingInteraction::Pressed => {
                *color = TileColor(Color::srgba(0.8, 0.8, 0.8, 1.0)); // Grey when pressed
            }
            PickingInteraction::None => {
                *color = TileColor(Color::WHITE);
            }
        }
    }
}
