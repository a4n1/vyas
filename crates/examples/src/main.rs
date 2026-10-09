mod camera;

use std::fs;

use vyas::prelude::*;

use crate::camera::NoClipCamera;

fn main() {
    let render_config = RenderConfigBuilder::new()
        .resolution(Resolution::Scale(0.3))
        .crt(true)
        .build();

    let camera = CameraConfig {
        position: WorldPosition {
            x: 59.115646,
            y: 132.52881,
            z: 182.13463,
        },
        looking_at: WorldPosition {
            x: 60.04423,
            y: 132.58475,
            z: 181.76775,
        },
        fov: 45.0,
    };
    let controller = NoClipCamera::new(&camera.position, &camera.looking_at);

    App::new()
        .set_camera(camera)
        .set_cursor_capture(true)
        .set_render_config(render_config)
        .set_egui(|ctx| {
            egui::Window::new("Controls").show(ctx, |ui| {
                ui.label("Hello from egui");
            });
        })
        .insert_resource(controller)
        .add_systems(Startup, load_scene)
        .add_systems(Startup, setup_lights)
        .add_systems(Update, NoClipCamera::update)
        .run();
}

fn load_scene(mut commands: Commands, mut palette: ColorPalette) {
    let path = "./crates/examples/assets/san-miguel.vox";
    let bytes = fs::read(path).expect("failed to read asset");
    let grid = vox::load_with_palette(&bytes, &mut palette).expect("failed to load vox model");

    commands.spawn(Scene { grid });
}

fn setup_lights(mut commands: Commands) {
    commands.spawn(AmbientLight {
        color: Color::Srgb(Srgb {
            r: 255,
            g: 255,
            b: 255,
        }),
        brightness: 0.25,
    });

    commands.spawn(DirectionalLight {
        color: Color::Srgb(Srgb {
            r: 255,
            g: 255,
            b: 255,
        }),
        illuminance: 0.75,
        direction: Direction {
            x: 0.425,
            y: -0.75,
            z: 0.5,
        },
    });
}
