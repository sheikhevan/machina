use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

pub struct BasicConveyorPlugin;

impl Plugin for BasicConveyorPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SpawnConveyorMsg>()
            .add_systems(Startup, setup_basic_conveyor)
            .add_systems(Update, spawn_conveyor_at_cursor);
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
