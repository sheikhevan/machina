use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::buildings::{basic_conveyor, oil_extractor};

pub struct DebugEguiPlugin;

impl Plugin for DebugEguiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_building_images)
            .add_systems(EguiPrimaryContextPass, debug_egui_menu);
    }
}

#[derive(Resource)]
struct BuildingImages {
    basic_conveyor: Handle<Image>,
    oil_extractor: Handle<Image>,
}

fn setup_building_images(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(BuildingImages {
        basic_conveyor: asset_server.load("textures/basic_conveyor.png"),
        oil_extractor: asset_server.load("textures/oil_extractor.png"),
    });
}

fn debug_egui_menu(
    mut contexts: EguiContexts,
    building_images: Res<BuildingImages>,
    time: Res<Time>,
    mut spawn_conveyor_writer: MessageWriter<basic_conveyor::SpawnConveyorMsg>,
    mut spawn_oil_extractor_writer: MessageWriter<oil_extractor::SpawnOilExtractorMsg>,
) -> Result {
    let fps = 10.0;

    let basic_conveyor_tid = contexts.add_image(bevy_egui::EguiTextureHandle::Strong(
        building_images.basic_conveyor.clone(),
    ));

    let oil_extractor_tid = contexts.add_image(bevy_egui::EguiTextureHandle::Strong(
        building_images.oil_extractor.clone(),
    ));

    egui::Window::new("DEBUG").show(contexts.ctx_mut()?, |ui| {
        ui.label("Buildings");

        // I really struggled trying to make this work with a list but I really couldn't so you get
        // to see me just put magic numbers everywhere and do it all manually. Yes I am a shit
        // coder thank you for asking.

        // Basic conveyor
        ui.collapsing("Basic Conveyor", |ui| {
            let num_frames = 5;

            let frame_index = ((time.elapsed_secs() * fps) as usize) % num_frames;

            let u_min = (frame_index as f32 * 32.0) / 160.0;
            let u_max = ((frame_index + 1) as f32 * 32.0) / 160.0;

            let uv = egui::Rect::from_min_max(egui::pos2(u_min, 0.0), egui::pos2(u_max, 1.0));

            let image = egui::Image::new(egui::load::SizedTexture::new(
                basic_conveyor_tid,
                egui::vec2(32.0, 32.0),
            ))
            .uv(uv);

            ui.add(image);

            if ui.button("Spawn").clicked() {
                spawn_conveyor_writer.write(basic_conveyor::SpawnConveyorMsg);
            }
        });

        // Oil extractor
        ui.collapsing("Oil Extractor", |ui| {
            let num_frames = 5;

            let frame_index = ((time.elapsed_secs() * fps) as usize) % num_frames;

            let u_min = (frame_index as f32 * 32.0) / 160.0;
            let u_max = ((frame_index + 1) as f32 * 32.0) / 160.0;

            let uv = egui::Rect::from_min_max(egui::pos2(u_min, 0.0), egui::pos2(u_max, 1.0));

            let image = egui::Image::new(egui::load::SizedTexture::new(
                oil_extractor_tid,
                egui::vec2(32.0, 32.0),
            ))
            .uv(uv);

            ui.add(image);

            if ui.button("Spawn").clicked() {
                spawn_oil_extractor_writer.write(oil_extractor::SpawnOilExtractorMsg);
            }
        });
    });
    Ok(())
}
