use crate::buildings::helpers::{BuildingRotation, snap_to_grid, world_to_grid};
use bevy::prelude::*;
use std::collections::HashMap;

pub struct PipePlugin;

impl Plugin for PipePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SpawnPipeMsg>()
            .init_resource::<PipeState>()
            .add_systems(Startup, setup_pipe)
            .add_systems(
                Update,
                (
                    start_pipe_preview,
                    update_pipe_preview,
                    rotate_pipe_preview,
                    place_pipe,
                    update_pipe_connections,
                )
                    .chain(),
            );
    }
}

#[derive(Message)]
pub struct SpawnPipeMsg;

#[derive(Resource)]
pub struct PipeAsset {
    pub texture: Handle<Image>,
    pub atlas_layout: Handle<TextureAtlasLayout>,
}

#[derive(Resource, Default)]
pub struct PipeState {
    pub placing: bool,
    pub preview: Option<Entity>,
    pub rotation: BuildingRotation,
}

#[derive(Component)]
pub struct Pipe;

#[derive(Component)]
pub struct PipePreview;

fn setup_pipe(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture = asset_server.load("textures/pipe.png");

    // Create texture atlas layout for 128x32 spritesheet with 4 frames of 32x32
    let layout = TextureAtlasLayout::from_grid(
        UVec2::new(32, 32), // Size of each frame
        4,                  // Number of columns
        1,                  // Number of rows
        None,
        None,
    );
    let atlas_layout = texture_atlases.add(layout);

    commands.insert_resource(PipeAsset {
        texture,
        atlas_layout,
    });
}

fn start_pipe_preview(
    mut commands: Commands,
    mut msg_reader: MessageReader<SpawnPipeMsg>,
    mut state: ResMut<PipeState>,
    pipe_asset: Res<PipeAsset>,
) {
    for _ in msg_reader.read() {
        state.placing = true;
        state.rotation = BuildingRotation::default(); // Reset thee rotation

        let preview = commands
            .spawn((
                PipePreview,
                Pipe,
                BuildingRotation::default(),
                Sprite {
                    image: pipe_asset.texture.clone(),
                    texture_atlas: Some(TextureAtlas {
                        layout: pipe_asset.atlas_layout.clone(),
                        index: 0,
                    }),
                    color: Color::srgba(1.0, 1.0, 1.0, 0.7), // Last value for preview opacity
                    ..default()
                },
                Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
            ))
            .id();

        state.preview = Some(preview);
    }
}

fn update_pipe_preview(
    state: Res<PipeState>,
    q_windows: Query<&Window>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    mut q_preview: Query<&mut Transform, With<PipePreview>>,
) {
    if !state.placing {
        return;
    }

    let Ok(window) = q_windows.single() else {
        return;
    };

    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    let Ok((camera, camera_transform)) = q_camera.single() else {
        return;
    };

    // Convert camera coords -> world coords
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else {
        return;
    };

    let snapped_pos = snap_to_grid(world_pos, 32.0);

    // Update the preview position
    if let Some(preview) = state.preview {
        if let Ok(mut transform) = q_preview.get_mut(preview) {
            transform.translation = snapped_pos.extend(10.0);
        }
    }
}

fn rotate_pipe_preview(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<PipeState>,
    mut q_preview: Query<(&mut BuildingRotation, &mut Transform), With<PipePreview>>,
) {
    if !state.placing {
        return;
    }

    if keyboard.just_pressed(KeyCode::KeyR) {
        // Rotate the state
        state.rotation.rotate_clockwise();

        // Rotate the preview
        if let Some(preview) = state.preview {
            if let Ok((mut rotation, mut transform)) = q_preview.get_mut(preview) {
                rotation.rotate_clockwise();
                transform.rotation = Quat::from_rotation_z(rotation.to_radians());
            }
        }
    }
}

fn place_pipe(
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut state: ResMut<PipeState>,
    pipe_asset: Res<PipeAsset>,
    q_preview: Query<(&Transform, &BuildingRotation), With<PipePreview>>,
) {
    if !state.placing {
        return;
    }

    if mouse_button.just_pressed(MouseButton::Left) {
        // Get the preview position
        if let Some(preview) = state.preview {
            if let Ok((preview_transform, rotation)) = q_preview.get(preview) {
                // Now we spawn the basic conveyor
                commands.spawn((
                    Pipe,
                    *rotation,
                    Sprite {
                        image: pipe_asset.texture.clone(),
                        texture_atlas: Some(TextureAtlas {
                            layout: pipe_asset.atlas_layout.clone(),
                            index: 0,
                        }),
                        ..default()
                    },
                    *preview_transform,
                ));
            }

            // And despawn the preview
            commands.entity(preview).despawn();
        }

        // Change state to exit placement mode
        state.placing = false;
        state.preview = None;
    }
}

fn update_pipe_connections(
    mut q_pipes: Query<
        (&mut Transform, &mut Sprite, &BuildingRotation),
        (With<Pipe>, Without<PipePreview>),
    >,
) {
    // We first build a map of grid positions to check for neighbors
    let mut pipe_positions: HashMap<(i32, i32), (BuildingRotation, usize)> = HashMap::new();
    let mut pipe_data: Vec<((i32, i32), BuildingRotation, usize)> = Vec::new();

    for (i, (transform, _, rotation)) in q_pipes.iter().enumerate() {
        let grid_pos = world_to_grid(transform.translation);
        pipe_positions.insert(grid_pos, (*rotation, i));
        pipe_data.push((grid_pos, *rotation, i));
    }

    // Now, we check each pipe and determine its texture index
    for (grid_pos, rotation, index) in pipe_data.iter() {
        let (x, y) = grid_pos;

        // Helper functions to check if a neighbor is vertical or horizontal
        let is_vertical = |pos: (i32, i32)| -> bool {
            pipe_positions
                .get(&pos)
                .map(|(rot, _)| {
                    rot.to_radians().abs() % std::f32::consts::PI > std::f32::consts::FRAC_PI_4
                })
                .unwrap_or(false)
        };

        let is_horizontal = |pos: (i32, i32)| -> bool {
            pipe_positions
                .get(&pos)
                .map(|(rot, _)| {
                    rot.to_radians().abs() % std::f32::consts::PI < std::f32::consts::FRAC_PI_4
                })
                .unwrap_or(false)
        };

        // Helper variables to check if current pipe is vertical or horizontal
        let current_is_vertical =
            rotation.to_radians().abs() % std::f32::consts::PI > std::f32::consts::FRAC_PI_4;
        let current_is_horizontal =
            rotation.to_radians().abs() % std::f32::consts::PI < std::f32::consts::FRAC_PI_4;

        // Check all 4 directions for neighboring pipes
        let has_above = pipe_positions.contains_key(&(*x, y + 1))
            && (is_vertical((*x, y + 1)) || current_is_vertical);
        let has_below = pipe_positions.contains_key(&(*x, y - 1))
            && (is_vertical((*x, y - 1)) || current_is_vertical);
        let has_left = pipe_positions.contains_key(&(*x - 1, *y))
            && (is_horizontal((*x - 1, *y)) || current_is_horizontal);
        let has_right = pipe_positions.contains_key(&(*x + 1, *y))
            && (is_horizontal((*x + 1, *y)) || current_is_horizontal);
        let has_vertical = has_above || has_below;
        let has_horizontal = has_left || has_right;

        let vertical_neighbor_is_vertical =
            (has_above && is_vertical((*x, y + 1))) || (has_below && is_vertical((*x, y - 1)));
        let horizontal_neighbor_is_horizontal =
            (has_left && is_horizontal((*x - 1, *y))) || (has_right && is_horizontal((*x + 1, *y)));

        let (texture_index, rotation_angle) = if has_left
            && has_right
            && has_above
            && has_below
            && vertical_neighbor_is_vertical
            && horizontal_neighbor_is_horizontal
        {
            (3, 0.0)
        } else if has_left && has_right && has_vertical && vertical_neighbor_is_vertical {
            // T-junction pipe (vertical connection)
            let angle = if has_below && is_vertical((*x, y - 1)) {
                std::f32::consts::PI // 180°
            } else {
                0.0 // 0°
            };
            (2, angle)
        } else if has_above && has_below && has_horizontal && horizontal_neighbor_is_horizontal {
            // T-junction pipe (horizontal connection)
            let angle = if has_right && is_horizontal((*x + 1, *y)) {
                std::f32::consts::FRAC_PI_2 // 90° - horizontal pipe to the right
            } else {
                -std::f32::consts::FRAC_PI_2 // 270° - horizontal pipe to the left
            };
            (2, angle)
        } else if has_vertical && has_horizontal {
            // Corner pipe
            let angle = if has_left && has_below {
                std::f32::consts::FRAC_PI_2 // 90°
            } else if has_right && has_below {
                std::f32::consts::PI // 180°
            } else if has_right && has_above {
                -std::f32::consts::FRAC_PI_2 // 270°
            } else if has_left && has_above {
                0.0 // 0°
            } else {
                0.0
            };
            (1, angle)
        } else {
            // Straight pipe
            (0, rotation.to_radians())
        };

        // Finally, update the sprite's texture index
        if let Some((mut transform, mut sprite, _)) = q_pipes.iter_mut().nth(*index) {
            if let Some(ref mut atlas) = sprite.texture_atlas {
                atlas.index = texture_index;
            }
            transform.rotation = Quat::from_rotation_z(rotation_angle);
        }
    }
}
