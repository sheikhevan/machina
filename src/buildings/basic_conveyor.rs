use crate::buildings::helpers::{BuildingRotation, snap_to_grid};
use bevy::prelude::*;

pub struct BasicConveyorPlugin;

impl Plugin for BasicConveyorPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SpawnConveyorMsg>()
            .init_resource::<BasicConveyorState>()
            .init_resource::<BasicConveyorAnimTimer>()
            .add_systems(Startup, setup_basic_conveyor)
            .add_systems(
                Update,
                (
                    start_basic_conveyor_preview,
                    update_basic_conveyor_preview,
                    rotate_basic_conveyor_preview,
                    place_basic_conveyor,
                    animate_basic_conveyors,
                )
                    .chain(),
            );
    }
}

#[derive(Message)]
pub struct SpawnConveyorMsg;

#[derive(Resource)]
pub struct BasicConveyorAsset {
    pub texture: Handle<Image>,
    pub atlas_layout: Handle<TextureAtlasLayout>,
}

#[derive(Resource, Default)]
pub struct BasicConveyorState {
    pub placing: bool,
    pub preview: Option<Entity>,
    pub rotation: BuildingRotation,
}

#[derive(Resource)]
struct BasicConveyorAnimTimer {
    timer: Timer,
    current_frame: usize,
}

impl Default for BasicConveyorAnimTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.1, TimerMode::Repeating),
            current_frame: 0,
        }
    }
}

#[derive(Component)]
pub struct BasicConveyor;

#[derive(Component)]
pub struct BasicConveyorPreview;

fn setup_basic_conveyor(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture = asset_server.load("textures/basic_conveyor.png");

    // Create texture atlas layout for 160x32 spritesheet with 5 frames of 32x32
    let layout = TextureAtlasLayout::from_grid(
        UVec2::new(32, 32), // Size of each frame
        5,                  // Number of columns
        1,                  // Number of rows
        None,
        None,
    );
    let atlas_layout = texture_atlases.add(layout);

    commands.insert_resource(BasicConveyorAsset {
        texture,
        atlas_layout,
    });
}

fn start_basic_conveyor_preview(
    mut commands: Commands,
    mut msg_reader: MessageReader<SpawnConveyorMsg>,
    mut state: ResMut<BasicConveyorState>,
    conveyor_asset: Res<BasicConveyorAsset>,
) {
    for _ in msg_reader.read() {
        state.placing = true;
        state.rotation = BuildingRotation::default(); // Reset thee rotation

        let preview = commands
            .spawn((
                BasicConveyorPreview,
                BasicConveyor,
                BuildingRotation::default(),
                Sprite {
                    image: conveyor_asset.texture.clone(),
                    texture_atlas: Some(TextureAtlas {
                        layout: conveyor_asset.atlas_layout.clone(),
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

fn update_basic_conveyor_preview(
    state: Res<BasicConveyorState>,
    q_windows: Query<&Window>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    mut q_preview: Query<&mut Transform, With<BasicConveyorPreview>>,
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

fn rotate_basic_conveyor_preview(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<BasicConveyorState>,
    mut q_preview: Query<(&mut BuildingRotation, &mut Transform), With<BasicConveyorPreview>>,
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

fn place_basic_conveyor(
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut state: ResMut<BasicConveyorState>,
    conveyor_asset: Res<BasicConveyorAsset>,
    q_preview: Query<(&Transform, &BuildingRotation), With<BasicConveyorPreview>>,
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
                    BasicConveyor,
                    *rotation,
                    Sprite {
                        image: conveyor_asset.texture.clone(),
                        texture_atlas: Some(TextureAtlas {
                            layout: conveyor_asset.atlas_layout.clone(),
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

fn animate_basic_conveyors(
    time: Res<Time>,
    mut q_sprite: Query<&mut Sprite, With<BasicConveyor>>,
    mut anim_timer: ResMut<BasicConveyorAnimTimer>,
) {
    anim_timer.timer.tick(time.delta());

    if anim_timer.timer.just_finished() {
        anim_timer.current_frame = (anim_timer.current_frame + 1) % 5;

        for mut sprite in q_sprite.iter_mut() {
            if let Some(ref mut atlas) = sprite.texture_atlas {
                atlas.index = anim_timer.current_frame;
            }
        }
    }
}
