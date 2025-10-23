use crate::buildings::helpers::{Building, BuildingRotation, snap_to_grid};
use bevy::prelude::*;

pub struct OilRefineryPlugin;

impl Plugin for OilRefineryPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SpawnOilRefineryMsg>()
            .init_resource::<OilRefineryState>()
            .init_resource::<OilRefineryAnimTimer>()
            .add_systems(Startup, setup_oil_refinery)
            .add_systems(
                Update,
                (
                    start_oil_refinery_preview,
                    update_oil_refinery_preview,
                    rotate_oil_refinery_preview,
                    place_oil_refinery,
                    animate_oil_refineries,
                )
                    .chain(),
            );
    }
}

#[derive(Message)]
pub struct SpawnOilRefineryMsg;

#[derive(Resource)]
pub struct OilRefineryAsset {
    pub texture: Handle<Image>,
    pub atlas_layout: Handle<TextureAtlasLayout>,
    pub rotation_indicator: Handle<Image>,
}

#[derive(Resource, Default)]
pub struct OilRefineryState {
    pub placing: bool,
    pub preview: Option<Entity>,
    pub rotation: BuildingRotation,
}

#[derive(Resource)]
struct OilRefineryAnimTimer {
    timer: Timer,
    current_frame: usize,
}

impl Default for OilRefineryAnimTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.1, TimerMode::Repeating),
            current_frame: 0,
        }
    }
}

#[derive(Component)]
pub struct OilRefinery;

#[derive(Component)]
pub struct OilRefineryPreview;

#[derive(Component)]
pub struct RotationIndicator;

fn setup_oil_refinery(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture = asset_server.load("textures/oil_refinery.png");
    let rotation_indicator = asset_server.load("textures/rotation_indicator.png");

    // Create texture atlas layout for 64x64 spritesheet with 5 frames of 64x64
    let layout = TextureAtlasLayout::from_grid(
        UVec2::new(64, 64), // Size of each frame
        5,                  // Number of columns
        1,                  // Number of rows
        None,
        None,
    );
    let atlas_layout = texture_atlases.add(layout);

    commands.insert_resource(OilRefineryAsset {
        texture,
        atlas_layout,
        rotation_indicator,
    });
}

fn start_oil_refinery_preview(
    mut commands: Commands,
    mut msg_reader: MessageReader<SpawnOilRefineryMsg>,
    mut state: ResMut<OilRefineryState>,
    oil_refinery_asset: Res<OilRefineryAsset>,
) {
    for _ in msg_reader.read() {
        state.placing = true;
        state.rotation = BuildingRotation::default(); // Reset thee rotation

        let preview = commands
            .spawn((
                OilRefineryPreview,
                OilRefinery,
                BuildingRotation::default(),
                Sprite {
                    image: oil_refinery_asset.texture.clone(),
                    texture_atlas: Some(TextureAtlas {
                        layout: oil_refinery_asset.atlas_layout.clone(),
                        index: 0,
                    }),
                    color: Color::srgba(1.0, 1.0, 1.0, 0.7), // Last value for preview opacity
                    ..default()
                },
                Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
            ))
            .with_children(|parent| {
                parent.spawn((
                    RotationIndicator,
                    Sprite {
                        image: oil_refinery_asset.rotation_indicator.clone(),
                        ..default()
                    },
                    Transform::from_xyz(8.0, 0.0, 1.0).with_rotation(Quat::from_rotation_z(0.0)),
                ));
            })
            .id();

        state.preview = Some(preview);
    }
}

fn update_oil_refinery_preview(
    state: Res<OilRefineryState>,
    q_windows: Query<&Window>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    mut q_preview: Query<&mut Transform, With<OilRefineryPreview>>,
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

    let snapped_pos = snap_to_grid(world_pos, 64.0);

    // Update the preview position
    if let Some(preview) = state.preview {
        if let Ok(mut transform) = q_preview.get_mut(preview) {
            transform.translation = snapped_pos.extend(10.0);
        }
    }
}

fn rotate_oil_refinery_preview(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<OilRefineryState>,
    q_preview: Query<&Children, With<OilRefineryPreview>>,
    mut q_preview_rotation: Query<&mut BuildingRotation, With<OilRefineryPreview>>,
    mut q_indicator: Query<&mut Transform, With<RotationIndicator>>,
) {
    if !state.placing {
        return;
    }

    if keyboard.just_pressed(KeyCode::KeyR) {
        // Rotate the state
        state.rotation.rotate_clockwise();

        // Update BuildingRotation but dont rotate the sprite itself
        if let Some(preview) = state.preview {
            if let Ok(mut rotation) = q_preview_rotation.get_mut(preview) {
                rotation.rotate_clockwise();
            }

            // Update the rotation indicator's position and rotation
            if let Ok(children) = q_preview.get(preview) {
                for child in children.iter() {
                    if let Ok(mut indicator_transform) = q_indicator.get_mut(child) {
                        let offset = state.rotation.to_direction() * 16.0;
                        indicator_transform.translation = offset.extend(1.0);

                        indicator_transform.rotation =
                            Quat::from_rotation_z(state.rotation.to_radians());
                    };
                }
            }
        }
    }
}

fn place_oil_refinery(
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut state: ResMut<OilRefineryState>,
    oil_refinery_asset: Res<OilRefineryAsset>,
    q_preview: Query<(&Transform, &BuildingRotation), With<OilRefineryPreview>>,
) {
    if !state.placing {
        return;
    }

    if mouse_button.just_pressed(MouseButton::Left) {
        // Get the preview position
        if let Some(preview) = state.preview {
            if let Ok((preview_transform, rotation)) = q_preview.get(preview) {
                // Now we spawn the oil refinery
                commands
                    .spawn((
                        Building,
                        OilRefinery,
                        Pickable::default(),
                        *rotation,
                        Sprite {
                            image: oil_refinery_asset.texture.clone(),
                            texture_atlas: Some(TextureAtlas {
                                layout: oil_refinery_asset.atlas_layout.clone(),
                                index: 0,
                            }),
                            ..default()
                        },
                        *preview_transform,
                    ))
                    .with_children(|parent| {
                        let offset = rotation.to_direction() * 8.0;
                        parent.spawn((
                            RotationIndicator,
                            Sprite {
                                image: oil_refinery_asset.rotation_indicator.clone(),
                                ..default()
                            },
                            Transform::from_translation(offset.extend(1.0))
                                .with_rotation(Quat::from_rotation_z(rotation.to_radians())),
                        ));
                    });
            }

            // And despawn the preview
            commands.entity(preview).despawn();
        }

        // Change state to exit placement mode
        state.placing = false;
        state.preview = None;
    }
}

fn animate_oil_refineries(
    time: Res<Time>,
    mut q_sprite: Query<&mut Sprite, With<OilRefinery>>,
    mut anim_timer: ResMut<OilRefineryAnimTimer>,
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
