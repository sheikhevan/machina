use bevy::{input::mouse::MouseWheel, math::ops::powf, prelude::*};

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

            max_zoom: 300.0,
            min_zoom: 10.0,
        }
    }
}

#[derive(Component)]
struct Speed(f32);

fn spawn_camera(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((Camera2d, Camera::default()));

    // This is just temporary for testing
    let shapes = [
        meshes.add(Circle::new(50.0)),
        meshes.add(CircularSector::new(50.0, 1.0)),
        meshes.add(CircularSegment::new(50.0, 1.25)),
        meshes.add(Ellipse::new(25.0, 50.0)),
        meshes.add(Annulus::new(25.0, 50.0)),
        meshes.add(Capsule2d::new(25.0, 50.0)),
        meshes.add(Rhombus::new(75.0, 100.0)),
        meshes.add(Rectangle::new(50.0, 100.0)),
        meshes.add(RegularPolygon::new(50.0, 6)),
        meshes.add(Triangle2d::new(
            Vec2::Y * 50.0,
            Vec2::new(-50.0, -50.0),
            Vec2::new(50.0, -50.0),
        )),
        meshes.add(Segment2d::new(
            Vec2::new(-50.0, 50.0),
            Vec2::new(50.0, -50.0),
        )),
        meshes.add(Polyline2d::new(vec![
            Vec2::new(-50.0, 50.0),
            Vec2::new(0.0, -50.0),
            Vec2::new(50.0, 50.0),
        ])),
    ];
    let num_shapes = shapes.len();

    for (i, shape) in shapes.into_iter().enumerate() {
        // Distribute colors evenly across the rainbow.
        let color = Color::hsl(360. * i as f32 / num_shapes as f32, 0.95, 0.7);

        commands.spawn((
            Mesh2d(shape),
            MeshMaterial2d(materials.add(color)),
            Transform::from_xyz(
                // Distribute shapes from -X_EXTENT/2 to +X_EXTENT/2.
                -900.0 / 2. + i as f32 / (num_shapes - 1) as f32 * 900.0,
                0.0,
                0.0,
            ),
        ));
    }
}

fn camera_controls(
    q_camera: Single<(&mut Camera, &mut Transform, &mut Projection)>,
    input: Res<ButtonInput<KeyCode>>,
    mut wheel_msg: MessageReader<MouseWheel>,
    time: Res<Time<Fixed>>,
) {
    let (mut camera, mut transform, mut projection) = q_camera.into_inner();

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
        .add_systems(Startup, spawn_camera)
        .add_systems(Update, camera_controls)
        .run();
}
