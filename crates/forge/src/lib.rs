use rand::prelude::*;
use vyas::prelude::*;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn web() {
    console_error_panic_hook::set_once();
    run();
}

pub fn run() {
    App::new()
        .set_camera(CameraConfig {
            position: WorldPosition {
                x: 0.0,
                y: 10.0,
                z: 20.0,
            },
            looking_at: WorldPosition {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            fov: 45.0,
        })
        .add_systems(Startup, draw_scene)
        .add_systems(Update, update_camera)
        .add_systems(Update, insert_voxel)
        .run();
}

fn draw_scene(mut voxel: VoxelCommands) {
    let mut rng = rand::rng();
    let colors = [
        Color::Srgb(Srgb {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }),
        Color::Srgb(Srgb {
            r: 0.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        }),
        Color::Srgb(Srgb {
            r: 0.0,
            g: 0.0,
            b: 1.0,
            a: 1.0,
        }),
    ];

    let size = 100;

    for i in (-8 * size)..(8 * size) {
        for j in (-8 * size)..(8 * size) {
            let color = colors.choose(&mut rng).unwrap().clone();

            voxel.spawn(GridPosition { x: i, y: 0, z: j }, Voxel { color });
        }
    }
}

fn update_camera(mut camera: Camera, input: Input) {
    const SPEED: f32 = 1.1;

    if input.pressed(InputButton::Key(KeyCode::ArrowUp)) {
        camera.position.z -= SPEED;
        camera.looking_at.z -= SPEED;
    }

    if input.pressed(InputButton::Key(KeyCode::ArrowDown)) {
        camera.position.z += SPEED;
        camera.looking_at.z += SPEED;
    }

    if input.pressed(InputButton::Key(KeyCode::ArrowLeft)) {
        camera.position.x -= SPEED;
        camera.looking_at.x -= SPEED;
    }

    if input.pressed(InputButton::Key(KeyCode::ArrowRight)) {
        camera.position.x += SPEED;
        camera.looking_at.x += SPEED;
    }
}

fn insert_voxel(input: Input, mut voxels: VoxelCommands) {
    if !input.just_pressed(InputButton::Mouse(MouseButton::Left)) {
        return;
    }

    let Some(hit) = input.voxel_hit() else {
        return;
    };

    voxels.spawn(
        hit.position.adjacent(hit.face),
        Voxel {
            color: Color::Srgb(Srgb {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            }),
        },
    );
}
