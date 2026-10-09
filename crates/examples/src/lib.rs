mod camera;
mod ui;

use std::{cell::RefCell, io::Read, rc::Rc};

use flate2::read::GzDecoder;
use vyas::prelude::*;

use crate::camera::NoClipCamera;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen(start))]
pub fn run() {
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
        .set_name(ui::APP_NAME)
        .set_camera(camera)
        .set_cursor_capture(true)
        .set_render_config(render_config)
        .set_egui(ui::draw)
        .insert_resource(controller)
        .insert_resource(SceneData::default())
        .add_systems(Startup, set_scene)
        .add_systems(Startup, setup_lights)
        .add_systems(Update, load_scene)
        .add_systems(Update, NoClipCamera::update)
        .run();
}

#[derive(Default)]
struct SceneData(Rc<RefCell<Option<VoxAsset>>>);

#[cfg(not(target_arch = "wasm32"))]
fn set_scene(data: Res<SceneData>) {
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/san-miguel.vox.gz"
    ))
    .expect("failed to read san-miguel.vox.gz");
    *data.0.borrow_mut() = Some(parse_scene(&bytes));
}

#[cfg(target_arch = "wasm32")]
fn set_scene(data: Res<SceneData>) {
    let asset = data.0.clone();
    wasm_bindgen_futures::spawn_local(async move {
        let response = gloo_net::http::Request::get("./assets/san-miguel.vox.gz")
            .send()
            .await
            .expect("failed to fetch san-miguel.vox.gz");
        assert!(response.ok(), "failed to fetch san-miguel.vox.gz");
        let bytes = response
            .binary()
            .await
            .expect("failed to read san-miguel.vox.gz response");
        *asset.borrow_mut() = Some(parse_scene(&bytes));
    });
}

fn parse_scene(bytes: &[u8]) -> VoxAsset {
    let mut decoded = Vec::new();
    let bytes = if bytes.starts_with(&[0x1f, 0x8b]) {
        GzDecoder::new(bytes)
            .read_to_end(&mut decoded)
            .expect("failed to decompress san-miguel.vox.gz");
        decoded.as_slice()
    } else {
        bytes
    };
    vox::load(bytes).expect("failed to load vox model")
}

fn load_scene(mut commands: Commands, mut palette: ColorPalette, data: Res<SceneData>) {
    let Some(scene) = data.0.borrow_mut().take() else {
        return;
    };

    *palette = scene.palette;
    commands.spawn(Scene { grid: scene.grid });

    #[cfg(target_arch = "wasm32")]
    if let Some(loader) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id("loader"))
    {
        let _ = loader.set_class_name("hidden");
    }
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
