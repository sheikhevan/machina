use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::buildings::helpers::{DeleteMode, delete_clicked_building};

use crate::buildings::{oil_container, oil_extractor, pipe};

pub struct DebugEguiPlugin;

impl Plugin for DebugEguiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DeleteMode>()
            .add_systems(Startup, setup_building_images)
            .add_systems(Update, delete_clicked_building)
            .add_systems(EguiPrimaryContextPass, debug_egui_menu);
    }
}

#[derive(Resource)]
struct BuildingImages {
    pipe: Handle<Image>,
    oil_extractor: Handle<Image>,
    oil_containers: Handle<Image>,
}

fn setup_building_images(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(BuildingImages {
        pipe: asset_server.load("textures/pipe.png"),
        oil_extractor: asset_server.load("textures/oil_extractor.png"),
        oil_containers: asset_server.load("textures/oil_container.png"),
    });
}

fn debug_egui_menu(
    mut contexts: EguiContexts,
    building_images: Res<BuildingImages>,
    time: Res<Time>,
    mut delete_mode: ResMut<DeleteMode>,
    mut spawn_pipe_writer: MessageWriter<pipe::SpawnPipeMsg>,
    mut spawn_oil_extractor_writer: MessageWriter<oil_extractor::SpawnOilExtractorMsg>,
    mut spawn_small_oil_container_writer: MessageWriter<oil_container::SpawnSmallOilContainerMsg>,
    mut spawn_medium_oil_container_writer: MessageWriter<oil_container::SpawnMediumOilContainerMsg>,
    mut spawn_large_oil_container_writer: MessageWriter<oil_container::SpawnLargeOilContainerMsg>,
) -> Result {
    let fps = 10.0;

    let pipe_tid = contexts.add_image(bevy_egui::EguiTextureHandle::Strong(
        building_images.pipe.clone(),
    ));

    let oil_extractor_tid = contexts.add_image(bevy_egui::EguiTextureHandle::Strong(
        building_images.oil_extractor.clone(),
    ));

    let oil_containers_tid = contexts.add_image(bevy_egui::EguiTextureHandle::Strong(
        building_images.oil_containers.clone(),
    ));

    egui::Window::new("DEBUG").show(contexts.ctx_mut()?, |ui| {
        ui.label("Tools");
        ui.checkbox(&mut delete_mode.active, "Delete Mode");
        ui.label("Buildings");

        // Pipe
        ui.collapsing("Pipe", |ui| {
            let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(0.25, 1.0));

            let image = egui::Image::new(egui::load::SizedTexture::new(
                pipe_tid,
                egui::vec2(32.0, 32.0),
            ))
            .uv(uv);

            ui.add(image);

            if ui.button("Spawn").clicked() {
                spawn_pipe_writer.write(pipe::SpawnPipeMsg);
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

        // Oil containers
        ui.collapsing("Oil Containers", |ui| {
            // Small container
            ui.label("Small");
            let uv_small = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(0.333, 1.0));

            let image_small = egui::Image::new(egui::load::SizedTexture::new(
                oil_containers_tid,
                egui::vec2(32.0, 32.0),
            ))
            .uv(uv_small);

            ui.add(image_small);

            if ui.button("Spawn").clicked() {
                spawn_small_oil_container_writer.write(oil_container::SpawnSmallOilContainerMsg);
            }

            ui.separator();

            // Medium container
            ui.label("Medium");
            let uv_medium =
                egui::Rect::from_min_max(egui::pos2(0.333, 0.0), egui::pos2(0.666, 1.0));

            let image_medium = egui::Image::new(egui::load::SizedTexture::new(
                oil_containers_tid,
                egui::vec2(32.0, 32.0),
            ))
            .uv(uv_medium);

            ui.add(image_medium);

            if ui.button("Spawn").clicked() {
                spawn_medium_oil_container_writer.write(oil_container::SpawnMediumOilContainerMsg);
            }

            ui.separator();

            // Large container
            ui.label("Large");
            let uv_large = egui::Rect::from_min_max(egui::pos2(0.666, 0.0), egui::pos2(1.0, 1.0));

            let image_large = egui::Image::new(egui::load::SizedTexture::new(
                oil_containers_tid,
                egui::vec2(32.0, 32.0),
            ))
            .uv(uv_large);

            ui.add(image_large);

            if ui.button("Spawn").clicked() {
                spawn_large_oil_container_writer.write(oil_container::SpawnLargeOilContainerMsg);
            }
        });
    });

    Ok(())
}
