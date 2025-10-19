use bevy::prelude::*;

pub fn snap_to_grid(world_pos: Vec2, tile_size: f32) -> Vec2 {
    Vec2::new(
        (world_pos.x / tile_size).floor() * tile_size + tile_size / 2.0,
        (world_pos.y / tile_size).floor() * tile_size + tile_size / 2.0,
    )
}
