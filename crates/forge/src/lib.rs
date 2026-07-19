mod history;
mod storage;

use std::{cell::RefCell, rc::Rc};

use vyas::prelude::*;
use wasm_bindgen::prelude::*;

use crate::history::{Action, History};

const GRID_SIZE: i32 = 64;

struct InsertPlane(Option<i32>);

struct CursorState(CursorMode);

#[wasm_bindgen(module = "/src/browser.js")]
extern "C" {
    #[wasm_bindgen(js_name = onGridUpdate)]
    fn on_grid_update();
}

struct Browser;

impl Browser {
    fn trigger_grid_update() {
        on_grid_update();
    }
}

#[derive(Clone, Default)]
struct SharedGrid(Rc<RefCell<Grid>>);

impl SharedGrid {
    fn new() -> Self {
        Self::default()
    }
}

#[wasm_bindgen]
pub enum CursorMode {
    Insert,
    Remove,
}

struct VoxelFile {
    file: Option<Vec<u8>>,
    initial_load: Option<bool>,
}

struct EditTransaction(Vec<Action>);

#[wasm_bindgen]
pub struct Forge {
    client: SharedClient,
    grid: SharedGrid,
}

#[wasm_bindgen]
impl Forge {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console_error_panic_hook::set_once();

        let client = SharedClient::new();
        let grid = SharedGrid::new();

        App::new()
            .set_client(Some(client.clone()))
            .set_camera(CameraConfig {
                position: WorldPosition {
                    x: -120.0,
                    y: 80.0,
                    z: -120.0,
                },
                looking_at: WorldPosition {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                fov: 45.0,
            })
            .insert_resource(Color::Srgb(Srgb { r: 0, g: 0, b: 0 }))
            .insert_resource(CursorState(CursorMode::Insert))
            .insert_resource(InsertPlane(Some(0)))
            .insert_resource(VoxelFile {
                file: None,
                initial_load: None,
            })
            .insert_resource(History::new())
            .insert_resource(EditTransaction(Vec::new()))
            .insert_resource(grid.clone())
            .add_systems(Startup, draw_scene)
            .add_systems(Update, update_camera_yaw)
            .add_systems(Update, update_camera_pitch)
            .add_systems(Update, update_camera_zoom)
            .add_systems(Update, update_camera_position)
            .add_systems(Update, edit_voxel)
            .add_systems(Update, load_file)
            .add_systems(Update, undo)
            .add_systems(Update, redo)
            .run();

        Self { client, grid }
    }

    pub fn set_color(&self, r: u8, g: u8, b: u8) {
        self.client
            .set_resource::<Color>(Color::Srgb(Srgb { r, g, b }));
    }

    pub fn set_cursor_mode(&self, cursor_mode: CursorMode) {
        self.client
            .set_resource::<CursorState>(CursorState(cursor_mode));
    }

    pub fn load_grid(&self, bytes: Vec<u8>, initial_load: Option<bool>) {
        self.client.set_resource::<VoxelFile>(VoxelFile {
            file: Some(bytes),
            initial_load: initial_load,
        });
    }

    pub fn export_grid(&self) -> Option<JsValue> {
        let grid = self.grid.0.borrow();
        let grid = Self::normalize_grid(&grid);

        let serialized_result = match serde_wasm_bindgen::to_value(&grid) {
            Ok(result) => Some(result),
            Err(e) => {
                log::error!("failed to serialize result: {e:#?}");
                None
            }
        };

        serialized_result
    }

    fn normalize_grid(grid: &Grid) -> Grid {
        grid.iter()
            .map(|(position, voxel)| {
                let position = GridPosition {
                    x: position.x + GRID_SIZE / 2,
                    y: position.y,
                    z: position.z + GRID_SIZE / 2,
                };
                (position, voxel.clone())
            })
            .collect()
    }

    fn denormalize_grid(grid: &Grid) -> Grid {
        grid.iter()
            .map(|(position, voxel)| {
                let position = GridPosition {
                    x: position.x - GRID_SIZE / 2,
                    y: position.y,
                    z: position.z - GRID_SIZE / 2,
                };
                (position, voxel.clone())
            })
            .collect()
    }

    fn grid_in_bounds(grid: &Grid) -> bool {
        grid.keys().all(|position| {
            (0..GRID_SIZE).contains(&position.x)
                && (0..GRID_SIZE).contains(&position.y)
                && (0..GRID_SIZE).contains(&position.z)
        })
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

fn edit_voxel(
    input: Input,
    mut voxels: VoxelCommands,
    color: Res<Color>,
    cursor_state: Res<CursorState>,
    mut edit_transaction: ResMut<EditTransaction>,
    mut insert_plane: ResMut<InsertPlane>,
    grid: Res<SharedGrid>,
    mut history: ResMut<History>,
) {
    if !input.pressed(InputButton::Mouse(MouseButton::Left)) {
        if !edit_transaction.0.is_empty() {
            history.insert_actions(edit_transaction.0.drain(..).collect())
        }

        insert_plane.0 = None;
        return;
    }

    let Some(hit) = input.voxel_hit() else {
        return;
    };

    match cursor_state.0 {
        CursorMode::Insert => insert_voxel(
            hit,
            &color,
            &mut insert_plane,
            &grid,
            &mut voxels,
            &mut edit_transaction,
        ),
        CursorMode::Remove => delete_voxel(hit, &grid, &mut voxels, &mut edit_transaction),
    }
}

fn insert_voxel(
    hit: &VoxelHit,
    color: &Color,
    insert_plane: &mut InsertPlane,
    grid: &SharedGrid,
    voxels: &mut VoxelCommands,
    edit_transaction: &mut EditTransaction,
) {
    let position = hit.position.adjacent(hit.face);

    if position.y < 0 || position.y >= GRID_SIZE {
        return;
    }

    if let Some(insert_plane) = insert_plane.0
        && insert_plane != position.y
    {
        return;
    }

    if position.x < -GRID_SIZE / 2 || position.x >= GRID_SIZE / 2 {
        return;
    }

    if position.z < -GRID_SIZE / 2 || position.z >= GRID_SIZE / 2 {
        return;
    }

    let voxel = Voxel {
        color: color.clone(),
    };

    if let Some(v) = grid.0.borrow_mut().get(&position)
        && v == &voxel
    {
        return;
    }

    insert_plane.0 = Some(position.y);

    spawn_voxel(&position, &voxel, grid, voxels);
    edit_transaction
        .0
        .push(Action::InsertVoxel { voxel, position });
    Browser::trigger_grid_update();
}

fn spawn_voxel(
    position: &GridPosition,
    voxel: &Voxel,
    grid: &SharedGrid,
    voxels: &mut VoxelCommands,
) {
    voxels.spawn(position.clone(), voxel.clone());
    grid.0.borrow_mut().insert(position.clone(), voxel.clone());
}

fn delete_voxel(
    hit: &VoxelHit,
    grid: &SharedGrid,
    voxels: &mut VoxelCommands,
    edit_transaction: &mut EditTransaction,
) {
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

    let Some(voxel) = despawn_voxel(&position, grid, voxels) else {
        return;
    };

    edit_transaction.0.push(Action::DeleteVoxel {
        voxel: voxel,
        position,
    });
    Browser::trigger_grid_update();
}

fn despawn_voxel(
    position: &GridPosition,
    grid: &SharedGrid,
    voxels: &mut VoxelCommands,
) -> Option<Voxel> {
    if let Some(voxel) = grid.0.borrow_mut().remove(&position) {
        voxels.despawn(position.clone());
        Some(voxel)
    } else {
        None
    }
}

fn load_file(
    mut voxel_file: ResMut<VoxelFile>,
    grid: ResMut<SharedGrid>,
    mut voxels: VoxelCommands,
    mut history: ResMut<History>,
) {
    let Some(ref file) = voxel_file.file.clone() else {
        return;
    };

    voxel_file.file = None;

    let asset = match VoxelAsset::from_bytes(file) {
        Ok(asset) => asset,
        Err(e) => {
            log::warn!("failed to load asset: {e:#?}");
            return;
        }
    };

    if !Forge::grid_in_bounds(&asset.grid) {
        log::warn!("failed to load asset: grid out of bounds");
        return;
    }

    if !voxel_file.initial_load.is_some_and(|v| v) {
        history.insert_actions(vec![Action::LoadGrid {
            prev_grid: Forge::normalize_grid(&grid.0.borrow()),
            next_grid: asset.grid.clone(),
        }]);
    }

    load_grid(asset, &grid, &mut voxels);
}

fn load_grid(asset: VoxelAsset, grid: &SharedGrid, voxels: &mut VoxelCommands) {
    let prev_grid = grid.0.borrow().clone().into_iter().collect::<Vec<_>>();

    for (position, _) in prev_grid {
        voxels.despawn(position);
    }

    voxels.spawn_asset(
        asset.clone(),
        &GridPosition {
            x: -GRID_SIZE / 2,
            y: 0,
            z: -GRID_SIZE / 2,
        },
    );

    *grid.0.borrow_mut() = Forge::denormalize_grid(&asset.grid);
    Browser::trigger_grid_update();
}

fn undo(
    mut history: ResMut<History>,
    input: Input,
    grid: ResMut<SharedGrid>,
    mut voxels: VoxelCommands,
) {
    if primary_modifier_pressed(&input)
        && !input.pressed(InputButton::Modifier(Modifier::Shift))
        && input.just_pressed(InputButton::Key(KeyCode::KeyZ))
    {
        let Some(actions) = history.undo() else {
            return;
        };

        for action in actions {
            match action {
                Action::InsertVoxel { position, .. } => {
                    despawn_voxel(&position, &grid, &mut voxels);
                }
                Action::DeleteVoxel { position, voxel } => {
                    spawn_voxel(&position, &voxel, &grid, &mut voxels);
                }
                Action::LoadGrid { prev_grid, .. } => {
                    load_grid(VoxelAsset { grid: prev_grid }, &grid, &mut voxels);
                }
            };
        }

        Browser::trigger_grid_update();
    }
}

fn redo(
    mut history: ResMut<History>,
    input: Input,
    grid: ResMut<SharedGrid>,
    mut voxels: VoxelCommands,
) {
    if primary_modifier_pressed(&input)
        && input.pressed(InputButton::Modifier(Modifier::Shift))
        && input.just_pressed(InputButton::Key(KeyCode::KeyZ))
    {
        let Some(actions) = history.redo() else {
            return;
        };

        for action in actions {
            match action {
                Action::InsertVoxel { position, voxel } => {
                    spawn_voxel(&position, &voxel, &grid, &mut voxels);
                }
                Action::DeleteVoxel { position, .. } => {
                    despawn_voxel(&position, &grid, &mut voxels);
                }
                Action::LoadGrid { next_grid, .. } => {
                    load_grid(VoxelAsset { grid: next_grid }, &grid, &mut voxels);
                }
            };
        }

        Browser::trigger_grid_update();
    }
}

fn primary_modifier_pressed(input: &Input) -> bool {
    if is_macos() {
        input.pressed(InputButton::Modifier(Modifier::Super))
    } else {
        input.pressed(InputButton::Modifier(Modifier::Control))
    }
}

fn is_macos() -> bool {
    wgpu::web_sys::window()
        .map(|window| window.navigator().platform())
        .is_some_and(|platform| platform.unwrap_or_default().starts_with("Mac"))
}
