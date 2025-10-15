use crate::tiles::picking::TilemapBackend;
use bevy::{input::mouse::MouseWheel, math::ops::powf, prelude::*};
use bevy_ecs_tilemap::TilemapPlugin;

mod tiles;

#[derive(Component)]
struct Speed(f32);

#[derive(Component)]
struct AnimationIndices {
    first: usize,
    last: usize,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

#[derive(Component)]
struct Camera {
    speed: Speed,

    max_zoom: f32,
    min_zoom: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            speed: Speed(300.0),

            max_zoom: 4.0,
            min_zoom: 0.3,
        }
    }
}

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(&AnimationIndices, &mut AnimationTimer, &mut Sprite)>,
) {
    for (indices, mut timer, mut sprite) in &mut query {
        timer.tick(time.delta());

        if timer.just_finished()
            && let Some(atlas) = &mut sprite.texture_atlas
        {
            atlas.index = if atlas.index == indices.last {
                indices.first
            } else {
                atlas.index + 1
            };
        }
    }
}

fn spawn_camera(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    commands.spawn((Camera2d, Camera::default()));

    // This is temporary to show the conveyor belt sprite is working
    let texture = asset_server.load("textures/basic_conveyor.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 5, 1, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    let animation_indices = AnimationIndices { first: 0, last: 4 };

    commands.spawn((
        Sprite::from_atlas_image(
            texture,
            TextureAtlas {
                layout: texture_atlas_layout,
                index: animation_indices.first,
            },
        ),
        Transform::from_scale(Vec3::splat(6.0)),
        animation_indices,
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
    ));
}

fn camera_controls(
    q_camera: Single<(&mut Camera, &mut Transform, &mut Projection)>,
    input: Res<ButtonInput<KeyCode>>,
    mut wheel_msg: MessageReader<MouseWheel>,
    time: Res<Time<Fixed>>,
) {
    let (camera, mut transform, mut projection) = q_camera.into_inner();

    let speed = camera.speed.0 * time.delta_secs();

    if input.pressed(KeyCode::KeyW) {
        transform.translation.y += speed;
    }
    if input.pressed(KeyCode::KeyS) {
        transform.translation.y -= speed;
    }
    if input.pressed(KeyCode::KeyA) {
        transform.translation.x -= speed;
    }
    if input.pressed(KeyCode::KeyD) {
        transform.translation.x += speed;
    }

    if let Projection::Orthographic(projection2d) = &mut *projection {
        for msg in wheel_msg.read() {
            if msg.y > 0.0 {
                projection2d.scale *= powf(4.0f32, time.delta_secs());
            }
            if msg.y < 0.0 {
                projection2d.scale *= powf(0.25f32, time.delta_secs());
            }
        }

        projection2d.scale = projection2d.scale.clamp(camera.min_zoom, camera.max_zoom);
    }
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: String::from("Machina 2D"),
                        ..Default::default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins((TilemapPlugin, TilemapBackend))
        .add_systems(Startup, spawn_camera)
        .add_systems(Update, camera_controls)
        .add_systems(Update, animate_sprite)
        .add_systems(Startup, tiles::tiles_startup)
        .run();
}
