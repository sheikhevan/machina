use crate::buildings::helpers::{BuildingRotation, snap_to_grid};
use bevy::prelude::*;

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

    // Create texture atlas layout for 32x32 spritesheet with 1 frame of 32x32
    let layout = TextureAtlasLayout::from_grid(
        UVec2::new(32, 32), // Size of each frame
        1,                  // Number of columns
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
