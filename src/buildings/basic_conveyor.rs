use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

pub struct BasicConveyorPlugin;

impl Plugin for BasicConveyorPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SpawnConveyorMsg>()
            .add_systems(Startup, setup_basic_conveyor)
            .add_systems(Update, (spawn_conveyor_at_cursor, animate_conveyors));
    }
}

#[derive(Message)]
pub struct SpawnConveyorMsg;

#[derive(Resource)]
pub struct BasicConveyorAsset {
    pub texture: Handle<Image>,
    pub atlas_layout: Handle<TextureAtlasLayout>,
}

#[derive(Component)]
pub struct BasicConveyor {
    animation_timer: Timer,
    current_frame: usize,
}

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

fn spawn_conveyor_at_cursor(
    mut commands: Commands,
    mut msg_reader: MessageReader<SpawnConveyorMsg>,
    conveyor_asset: Res<BasicConveyorAsset>,
    q_windows: Query<&Window>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
) {
    for _ in msg_reader.read() {
        let window = q_windows.single().unwrap();

        let Some(cursor_pos) = window.cursor_position() else {
            continue;
        };

        let (camera, camera_transform) = q_camera.single().unwrap();

        // Convert cursor position -> world coordinates
        let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else {
            continue;
        };

        commands.spawn((
            BasicConveyor {
                animation_timer: Timer::from_seconds(0.1, TimerMode::Repeating),
                current_frame: 0,
            },
            Sprite {
                image: conveyor_asset.texture.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: conveyor_asset.atlas_layout.clone(),
                    index: 0,
                }),
                ..default()
            },
            Transform::from_translation(world_pos.extend(10.0)),
        ));
    }
}

fn animate_conveyors(time: Res<Time>, mut q_sprite: Query<(&mut BasicConveyor, &mut Sprite)>) {
    for (mut conveyor, mut sprite) in q_sprite.iter_mut() {
        conveyor.animation_timer.tick(time.delta());

        if conveyor.animation_timer.just_finished() {
            conveyor.current_frame = (conveyor.current_frame + 1) % 5;
            if let Some(ref mut atlas) = sprite.texture_atlas {
                atlas.index = conveyor.current_frame;
            }
        }
    }
}
