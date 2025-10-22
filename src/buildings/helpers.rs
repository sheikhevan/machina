use bevy::prelude::*;

#[derive(Component)]
pub struct Building;

pub fn snap_to_grid(world_pos: Vec2, tile_size: f32) -> Vec2 {
    Vec2::new(
        (world_pos.x / tile_size).floor() * tile_size + tile_size / 2.0,
        (world_pos.y / tile_size).floor() * tile_size + tile_size / 2.0,
    )
}

pub fn world_to_grid(world_pos: Vec3) -> (i32, i32) {
    let tile_size = 32.0;
    (
        (world_pos.x / tile_size).round() as i32,
        (world_pos.y / tile_size).round() as i32,
    )
}

#[derive(Resource, Default)]
pub struct DeleteMode {
    pub active: bool,
}

pub fn check_if_clicked_building<'a>(
    mut msg_reader: MessageReader<Pointer<Click>>,
    q_buildings: Query<&Building>,
) -> Option<Entity> {
    for msg in msg_reader.read() {
        if q_buildings.contains(msg.entity) {
            return Some(msg.entity);
        }
    }
    None
}

pub fn delete_clicked_building(
    mut commands: Commands,
    mut msg_reader: MessageReader<Pointer<Click>>,
    q_buildings: Query<&Building>,
    delete_mode: Res<DeleteMode>,
) {
    if !delete_mode.active {
        return;
    }

    for msg in msg_reader.read() {
        if q_buildings.contains(msg.entity) {
            commands.entity(msg.entity).despawn();
        }
    }
}

// Rotation stuff
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BuildingRotation {
    North,
    #[default]
    East,
    South,
    West,
}

impl BuildingRotation {
    pub fn rotate_clockwise(&mut self) {
        *self = match self {
            BuildingRotation::North => BuildingRotation::East,
            BuildingRotation::East => BuildingRotation::South,
            BuildingRotation::South => BuildingRotation::West,
            BuildingRotation::West => BuildingRotation::North,
        };
    }

    pub fn to_radians(&self) -> f32 {
        match self {
            BuildingRotation::North => std::f32::consts::FRAC_PI_2,
            BuildingRotation::East => 0.0,
            BuildingRotation::South => -std::f32::consts::FRAC_PI_2,
            BuildingRotation::West => std::f32::consts::PI,
        }
    }

    pub fn to_direction(&self) -> Vec2 {
        match self {
            BuildingRotation::North => Vec2::new(0.0, 1.0),
            BuildingRotation::East => Vec2::new(1.0, 0.0),
            BuildingRotation::South => Vec2::new(0.0, -1.0),
            BuildingRotation::West => Vec2::new(-1.0, 0.0),
        }
    }
}
