mod camera;

use std::fs;

use vyas::prelude::*;

use crate::camera::NoClipCamera;

fn main() {
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
        .insert_resource(controller)
        .add_systems(Startup, load_scene)
        .add_systems(Update, NoClipCamera::update)
        .run();
}

fn load_scene(mut scene: Scene, mut palette: ColorPalette) {
    let path = "./crates/examples/assets/san-miguel.vox";
    let bytes = fs::read(path).expect("failed to read asset");
    let grid = vox::load_with_palette(&bytes, &mut palette).expect("failed to load vox model");

    scene.spawn(grid);
}
