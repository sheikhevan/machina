use crate::buildings::helpers::{Building, BuildingRotation, snap_to_grid};
use bevy::prelude::*;

pub struct OilContainerPlugin;

impl Plugin for OilContainerPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SpawnSmallOilContainerMsg>()
            .add_message::<SpawnMediumOilContainerMsg>()
            .add_message::<SpawnLargeOilContainerMsg>()
            .init_resource::<OilContainerState>()
            .add_systems(Startup, setup_oil_containers)
            .add_systems(
                Update,
                (
                    start_oil_container_preview,
                    update_oil_container_preview,
                    rotate_oil_container_preview,
                    place_oil_container,
                )
                    .chain(),
            );
    }
}

#[derive(Message)]
pub struct SpawnSmallOilContainerMsg;

#[derive(Message)]
pub struct SpawnMediumOilContainerMsg;

#[derive(Message)]
pub struct SpawnLargeOilContainerMsg;

#[derive(Resource)]
pub struct OilContainerAssets {
    pub texture: Handle<Image>,
    pub atlas_layout: Handle<TextureAtlasLayout>,
    pub rotation_indicator: Handle<Image>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ContainerSize {
    Small,
    Medium,
    Large,
}

impl ContainerSize {
    pub fn atlas_index(&self) -> usize {
        match self {
            ContainerSize::Small => 0,
            ContainerSize::Medium => 1,
            ContainerSize::Large => 2,
        }
    }

    pub fn indicator_offset(&self) -> f32 {
        match self {
            ContainerSize::Small => 8.0,
            ContainerSize::Medium => 16.0,
            ContainerSize::Large => 16.0,
        }
    }
}

#[derive(Resource, Default)]
pub struct OilContainerState {
    pub placing: bool,
    pub preview: Option<Entity>,
    pub rotation: BuildingRotation,
    pub size: Option<ContainerSize>,
}

#[derive(Component)]
pub struct SmallOilContainer;

#[derive(Component)]
pub struct MediumOilContainer;

#[derive(Component)]
pub struct LargeOilContainer;

#[derive(Component)]
pub struct OilContainerPreview;

#[derive(Component)]
pub struct RotationIndicator;

fn setup_oil_containers(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture = asset_server.load("textures/oil_container.png");
    let rotation_indicator = asset_server.load("textures/rotation_indicator.png");

    let layout = TextureAtlasLayout::from_grid(
        UVec2::new(32, 32),
        3, // 3 columns
        1, // 1 row
        None,
        None,
    );

    let atlas_layout = texture_atlases.add(layout);

    commands.insert_resource(OilContainerAssets {
        texture,
        atlas_layout,
        rotation_indicator,
    });
}

fn start_oil_container_preview(
    mut commands: Commands,
    mut small_msg: MessageReader<SpawnSmallOilContainerMsg>,
    mut medium_msg: MessageReader<SpawnMediumOilContainerMsg>,
    mut large_msg: MessageReader<SpawnLargeOilContainerMsg>,
    mut state: ResMut<OilContainerState>,
    assets: Res<OilContainerAssets>,
) {
    let size = if small_msg.read().next().is_some() {
        Some(ContainerSize::Small)
    } else if medium_msg.read().next().is_some() {
        Some(ContainerSize::Medium)
    } else if large_msg.read().next().is_some() {
        Some(ContainerSize::Large)
    } else {
        None
    };

    if let Some(container_size) = size {
        state.placing = true;
        state.rotation = BuildingRotation::default();
        state.size = Some(container_size);

        let preview = commands
            .spawn((
                OilContainerPreview,
                BuildingRotation::default(),
                Sprite {
                    image: assets.texture.clone(),
                    texture_atlas: Some(TextureAtlas {
                        layout: assets.atlas_layout.clone(),
                        index: container_size.atlas_index(),
                    }),
                    color: Color::srgba(1.0, 1.0, 1.0, 0.7),
                    ..default()
                },
                Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
            ))
            .with_children(|parent| {
                parent.spawn((
                    RotationIndicator,
                    Sprite {
                        image: assets.rotation_indicator.clone(),
                        ..default()
                    },
                    Transform::from_xyz(container_size.indicator_offset(), 0.0, 1.0),
                ));
            })
            .id();

        state.preview = Some(preview);
    }
}

fn update_oil_container_preview(
    state: Res<OilContainerState>,
    q_windows: Query<&Window>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    mut q_preview: Query<&mut Transform, With<OilContainerPreview>>,
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
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else {
        return;
    };

    let snapped_pos = snap_to_grid(world_pos, 32.0);

    if let Some(preview) = state.preview {
        if let Ok(mut transform) = q_preview.get_mut(preview) {
            transform.translation = snapped_pos.extend(10.0);
        }
    }
}

fn rotate_oil_container_preview(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<OilContainerState>,
    q_preview: Query<&Children, With<OilContainerPreview>>,
    mut q_preview_rotation: Query<&mut BuildingRotation, With<OilContainerPreview>>,
    mut q_indicator: Query<&mut Transform, With<RotationIndicator>>,
) {
    if !state.placing {
        return;
    }

    if keyboard.just_pressed(KeyCode::KeyR) {
        state.rotation.rotate_clockwise();

        if let Some(preview) = state.preview {
            if let Ok(mut rotation) = q_preview_rotation.get_mut(preview) {
                rotation.rotate_clockwise();
            }

            let indicator_offset = state.size.map(|s| s.indicator_offset()).unwrap_or(8.0);

            if let Ok(children) = q_preview.get(preview) {
                for child in children.iter() {
                    if let Ok(mut indicator_transform) = q_indicator.get_mut(child) {
                        let offset = state.rotation.to_direction() * indicator_offset;
                        indicator_transform.translation = offset.extend(1.0);
                        indicator_transform.rotation =
                            Quat::from_rotation_z(state.rotation.to_radians());
                    }
                }
            }
        }
    }
}

fn place_oil_container(
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut state: ResMut<OilContainerState>,
    assets: Res<OilContainerAssets>,
    q_preview: Query<(&Transform, &BuildingRotation), With<OilContainerPreview>>,
) {
    if !state.placing {
        return;
    }

    if mouse_button.just_pressed(MouseButton::Left) {
        if let Some(preview) = state.preview {
            if let Ok((preview_transform, rotation)) = q_preview.get(preview) {
                let size = state.size.unwrap_or(ContainerSize::Small);

                let entity = commands
                    .spawn((
                        Building,
                        Pickable::default(),
                        *rotation,
                        Sprite {
                            image: assets.texture.clone(),
                            texture_atlas: Some(TextureAtlas {
                                layout: assets.atlas_layout.clone(),
                                index: size.atlas_index(),
                            }),
                            ..default()
                        },
                        *preview_transform,
                    ))
                    .with_children(|parent| {
                        let offset = rotation.to_direction() * size.indicator_offset();
                        parent.spawn((
                            RotationIndicator,
                            Sprite {
                                image: assets.rotation_indicator.clone(),
                                ..default()
                            },
                            Transform::from_translation(offset.extend(1.0))
                                .with_rotation(Quat::from_rotation_z(rotation.to_radians())),
                        ));
                    })
                    .id();

                // Add the specific container component
                match size {
                    ContainerSize::Small => commands.entity(entity).insert(SmallOilContainer),
                    ContainerSize::Medium => commands.entity(entity).insert(MediumOilContainer),
                    ContainerSize::Large => commands.entity(entity).insert(LargeOilContainer),
                };

                commands.entity(preview).despawn();
            }
        }

        state.placing = false;
        state.preview = None;
        state.size = None;
    }
}
