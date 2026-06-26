use std::collections::{HashMap, HashSet};

use glam::{Vec3, Vec4};
use wgpu::SurfaceConfiguration;
use winit::dpi::PhysicalPosition;

use crate::{
    camera::CameraState,
    dda::{DDA, Ray, VoxelHit},
    ecs::{Res, World},
};

pub type Input<'a> = Res<'a, InputState>;

pub type KeyCode = winit::keyboard::KeyCode;

pub type MouseButton = winit::event::MouseButton;

pub type MousePosition = PhysicalPosition<f64>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputButton {
    Key(KeyCode),
    Mouse(MouseButton),
}

impl From<KeyCode> for InputButton {
    fn from(key: KeyCode) -> Self {
        Self::Key(key)
    }
}

impl From<MouseButton> for InputButton {
    fn from(button: MouseButton) -> Self {
        Self::Mouse(button)
    }
}

#[derive(Default)]
pub struct InputState {
    pressed: HashMap<InputButton, bool>,
    just_pressed: HashSet<InputButton>,
    mouse_position: MousePosition,
    voxel_hit: Option<VoxelHit>,
}

impl InputState {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn insert_pressed(&mut self, input_button: InputButton, pressed: bool) {
        if pressed {
            self.just_pressed.insert(input_button);
        }

        self.pressed.insert(input_button, pressed);
    }

    pub(crate) fn set_mouse_position(
        &mut self,
        mouse_position: MousePosition,
        world: &World,
        surface_config: &SurfaceConfiguration,
    ) {
        self.mouse_position = mouse_position;

        let camera = world.resource::<CameraState>();
        let ray = Self::ray_from_cursor(mouse_position, &camera, surface_config);
        self.voxel_hit = DDA::hit(&ray, world);
    }

    pub fn mouse_position(&self) -> MousePosition {
        self.mouse_position
    }

    pub fn update(&mut self) {
        self.just_pressed.clear();
    }

    pub fn pressed(&self, input_button: InputButton) -> bool {
        self.pressed.get(&input_button).is_some_and(|v| *v)
    }

    pub fn just_pressed(&self, input_button: InputButton) -> bool {
        self.just_pressed.contains(&input_button)
    }

    pub fn voxel_hit(&self) -> Option<VoxelHit> {
        self.voxel_hit.clone()
    }

    fn ray_from_cursor(
        mouse_position: MousePosition,
        camera: &Res<CameraState>,
        surface_config: &SurfaceConfiguration,
    ) -> Ray {
        let camera_position = Vec3::from(&camera.position);
        let view_projection = camera.build_view_projection_matrix();

        let ndc_x = mouse_position.x as f32 / surface_config.width as f32 * 2.0 - 1.0;
        let ndc_y = 1.0 - mouse_position.y as f32 / surface_config.height as f32 * 2.0;
        let world_point = view_projection.inverse() * Vec4::new(ndc_x, ndc_y, 1.0, 1.0);
        let world_point = world_point.truncate() / world_point.w;

        let direction = (world_point - camera_position).normalize_or_zero();

        Ray {
            origin: camera_position.into(),
            direction: direction.into(),
        }
    }
}
