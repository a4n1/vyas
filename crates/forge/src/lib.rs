use std::sync::{LazyLock, Mutex};

use vyas::prelude::*;

use wasm_bindgen::prelude::*;

const GRID_SIZE: i32 = 64;

#[wasm_bindgen]
pub fn init() {
    console_error_panic_hook::set_once();

    App::new()
        .set_camera(CameraConfig {
            position: WorldPosition {
                x: -120.0,
                y: 240.0,
                z: -120.0,
            },
            looking_at: WorldPosition {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            fov: 45.0,
        })
        .add_systems(Startup, draw_scene)
        .add_systems(Update, update_camera_yaw)
        .add_systems(Update, update_camera_pitch)
        .add_systems(Update, update_camera_zoom)
        .add_systems(Update, update_camera_position)
        .add_systems(Update, edit_voxel)
        .run();
}

static STATE: LazyLock<Mutex<State>> = LazyLock::new(|| {
    Mutex::new(State {
        color: VoxelColor(0x000000).into(),
        insert_plane: None,
        cursor_mode: CursorMode::Insert,
    })
});

#[derive(Clone, Copy)]
struct VoxelColor(u32);

#[derive(Clone, Copy)]
struct InsertPlane(i32);

#[wasm_bindgen]
pub enum CursorMode {
    Insert,
    Remove,
}

struct State {
    color: VoxelColor,
    insert_plane: Option<InsertPlane>,
    cursor_mode: CursorMode,
}

impl From<VoxelColor> for Color {
    fn from(value: VoxelColor) -> Self {
        let r = ((value.0 >> 16) & 0xFF) as u8;
        let g = ((value.0 >> 8) & 0xFF) as u8;
        let b = (value.0 & 0xFF) as u8;

        Color::Srgb(Srgb { r, g, b })
    }
}

impl From<Color> for VoxelColor {
    fn from(value: Color) -> Self {
        let r = value.r() as u32;
        let g = value.g() as u32;
        let b = value.b() as u32;

        let color = ((r << 16) | (g << 8) | b) as u32;

        VoxelColor(color)
    }
}

fn draw_scene(mut voxel: VoxelCommands) {
    for i in (-GRID_SIZE / 2)..(GRID_SIZE / 2) {
        for j in (-GRID_SIZE / 2)..(GRID_SIZE / 2) {
            voxel.spawn(
                GridPosition { x: i, y: -1, z: j },
                Voxel {
                    color: Color::Srgb(Srgb {
                        r: 128,
                        g: 128,
                        b: 128,
                    }),
                },
            );
        }
    }
}

fn update_camera_yaw(mut camera: Camera, input: Input) {
    const YAW_SPEED: f32 = 0.04;

    let mut yaw = 0.0;

    if input.pressed(InputButton::Key(KeyCode::ArrowLeft)) {
        yaw += YAW_SPEED;
    }

    if input.pressed(InputButton::Key(KeyCode::ArrowRight)) {
        yaw -= YAW_SPEED;
    }

    if yaw != 0.0 {
        let (sin, cos) = yaw.sin_cos();
        let x = camera.position.x - camera.looking_at.x;
        let z = camera.position.z - camera.looking_at.z;

        camera.position.x = camera.looking_at.x + x * cos - z * sin;
        camera.position.z = camera.looking_at.z + x * sin + z * cos;
    }
}

fn update_camera_pitch(mut camera: Camera, input: Input) {
    const PITCH_SPEED: f32 = 0.04;
    const MIN_PITCH: f32 = 0.05;
    const MAX_PITCH: f32 = 1.45;

    let mut pitch_delta = 0.0;

    if input.pressed(InputButton::Key(KeyCode::ArrowUp)) {
        pitch_delta += PITCH_SPEED;
    }

    if input.pressed(InputButton::Key(KeyCode::ArrowDown)) {
        pitch_delta -= PITCH_SPEED;
    }

    if pitch_delta != 0.0 {
        let offset_x = camera.position.x - camera.looking_at.x;
        let offset_y = camera.position.y - camera.looking_at.y;
        let offset_z = camera.position.z - camera.looking_at.z;
        let horizontal = (offset_x * offset_x + offset_z * offset_z).sqrt();

        let distance = (horizontal * horizontal + offset_y * offset_y).sqrt();

        if horizontal > 0.0 && distance > 0.0 {
            let current_pitch = offset_y.atan2(horizontal);
            let new_pitch = (current_pitch + pitch_delta).clamp(MIN_PITCH, MAX_PITCH);

            let new_horizontal = distance * new_pitch.cos();
            let new_y = distance * new_pitch.sin();

            let scale = new_horizontal / horizontal;

            camera.position.x = camera.looking_at.x + offset_x * scale;
            camera.position.z = camera.looking_at.z + offset_z * scale;
            camera.position.y = camera.looking_at.y + new_y;
        }
    }
}

fn update_camera_zoom(mut camera: Camera, input: Input) {
    const ZOOM_STEP_SCALE: f32 = 0.9;
    const PIXELS_PER_ZOOM_STEP: f32 = 50.0;
    const LINES_PER_ZOOM_STEP: f32 = 3.0;

    if let Some(scroll_delta) = input.scroll_delta() {
        let zoom_steps = match scroll_delta {
            ScrollDelta::LineDelta(_, y) => *y / LINES_PER_ZOOM_STEP,
            ScrollDelta::PixelDelta(position) => position.y as f32 / PIXELS_PER_ZOOM_STEP,
        };

        if zoom_steps != 0.0 {
            let scale = ZOOM_STEP_SCALE.powf(zoom_steps);
            let offset_x = camera.position.x - camera.looking_at.x;
            let offset_y = camera.position.y - camera.looking_at.y;
            let offset_z = camera.position.z - camera.looking_at.z;

            camera.position.x = camera.looking_at.x + offset_x * scale;
            camera.position.y = camera.looking_at.y + offset_y * scale;
            camera.position.z = camera.looking_at.z + offset_z * scale;
        }
    }
}

fn update_camera_position(mut camera: Camera, input: Input) {
    const PAN_SPEED_SCALE: f32 = 0.005;

    let forward_x = camera.looking_at.x - camera.position.x;
    let forward_y = camera.looking_at.y - camera.position.y;
    let forward_z = camera.looking_at.z - camera.position.z;
    let forward_length = (forward_x * forward_x + forward_z * forward_z).sqrt();
    let camera_distance =
        (forward_x * forward_x + forward_y * forward_y + forward_z * forward_z).sqrt();

    if forward_length == 0.0 || camera_distance == 0.0 {
        return;
    }

    let forward_x = forward_x / forward_length;
    let forward_z = forward_z / forward_length;
    let right_x = -forward_z;
    let right_z = forward_x;

    let mut delta_x = 0.0;
    let mut delta_z = 0.0;

    if input.pressed(InputButton::Key(KeyCode::KeyW)) {
        delta_x += forward_x;
        delta_z += forward_z;
    }

    if input.pressed(InputButton::Key(KeyCode::KeyS)) {
        delta_x -= forward_x;
        delta_z -= forward_z;
    }

    if input.pressed(InputButton::Key(KeyCode::KeyA)) {
        delta_x += right_x;
        delta_z += right_z;
    }

    if input.pressed(InputButton::Key(KeyCode::KeyD)) {
        delta_x -= right_x;
        delta_z -= right_z;
    }

    let delta_length = (delta_x * delta_x + delta_z * delta_z).sqrt();

    if delta_length == 0.0 {
        return;
    }

    let pan_speed = camera_distance * PAN_SPEED_SCALE;
    let delta_x = delta_x / delta_length * pan_speed;
    let delta_z = delta_z / delta_length * pan_speed;

    camera.position.x += delta_x;
    camera.position.z += delta_z;
    camera.looking_at.x += delta_x;
    camera.looking_at.z += delta_z;
}

#[wasm_bindgen]
pub fn set_color(color: u32) {
    if let Ok(mut lock) = STATE.lock() {
        lock.color = VoxelColor(color);
    }
}

#[wasm_bindgen]
pub fn set_cursor_mode(cursor_mode: CursorMode) {
    if let Ok(mut lock) = STATE.lock() {
        lock.cursor_mode = cursor_mode;
    }
}

fn edit_voxel(input: Input, voxels: VoxelCommands) {
    let Ok(mut lock) = STATE.lock() else {
        log::warn!("failed to take state lock");
        return;
    };

    if !input.pressed(InputButton::Mouse(MouseButton::Left)) {
        lock.insert_plane = None;
        return;
    }

    let Some(hit) = input.voxel_hit() else {
        return;
    };

    match lock.cursor_mode {
        CursorMode::Insert => insert_voxel(hit, lock.color, &mut lock.insert_plane, voxels),
        CursorMode::Remove => remove_voxel(hit, voxels),
    }
}

fn insert_voxel(
    hit: &VoxelHit,
    color: VoxelColor,
    insert_plane: &mut Option<InsertPlane>,
    mut voxels: VoxelCommands,
) {
    let position = hit.position.adjacent(hit.face);

    if position.y < 0 || position.y >= GRID_SIZE {
        return;
    }

    if let Some(insert_plane) = insert_plane
        && insert_plane.0 != position.y
    {
        return;
    }

    if position.x < -GRID_SIZE / 2 || position.x >= GRID_SIZE / 2 {
        return;
    }

    if position.z < -GRID_SIZE / 2 || position.z >= GRID_SIZE / 2 {
        return;
    }

    *insert_plane = Some(InsertPlane(position.y));

    voxels.spawn(
        position,
        Voxel {
            color: color.into(),
        },
    );
}

fn remove_voxel(hit: &VoxelHit, mut voxels: VoxelCommands) {
    let position = hit.position.clone();

    if position.y < 0 || position.y >= GRID_SIZE {
        return;
    }

    if position.x < -GRID_SIZE / 2 || position.x >= GRID_SIZE / 2 {
        return;
    }

    if position.z < -GRID_SIZE / 2 || position.z >= GRID_SIZE / 2 {
        return;
    }

    voxels.despawn(position);
}
