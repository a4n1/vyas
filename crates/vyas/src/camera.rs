use glam::{Mat4, Vec3};

use crate::ecs::ResMut;
use crate::prelude::WorldPosition;

pub type Camera<'a> = ResMut<'a, CameraState>;

#[derive(Debug, Clone)]
pub struct CameraConfig {
    pub position: WorldPosition,
    pub looking_at: WorldPosition,
    pub fov: f32,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            position: WorldPosition {
                x: 0.0,
                y: 1.0,
                z: 2.0,
            },
            looking_at: WorldPosition {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            fov: 45.0,
        }
    }
}

pub struct CameraState {
    pub position: WorldPosition,
    pub looking_at: WorldPosition,
    pub fovy: f32,
    pub(crate) aspect: f32,
}

impl CameraState {
    pub fn new(camera_config: CameraConfig) -> Self {
        Self {
            position: camera_config.position,
            looking_at: camera_config.looking_at,
            fovy: camera_config.fov,
            aspect: 1.0,
        }
    }

    pub(crate) fn build_view_projection_matrix(&self) -> Mat4 {
        let view = Mat4::look_at_rh(
            (&self.position).into(),
            (&self.looking_at).into(),
            Vec3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
        );
        let proj = Self::perspective_reverse_z(self.fovy.to_radians(), self.aspect, 0.1);

        proj * view
    }

    pub(crate) fn resize(&mut self, surface_config: &wgpu::SurfaceConfiguration) {
        self.aspect = surface_config.width as f32 / surface_config.height as f32;
    }

    #[rustfmt::skip]
    fn perspective_reverse_z(fovy: f32, aspect: f32, znear: f32) -> Mat4 {
        let f = 1.0 / (0.5 * fovy).tan();

        Mat4::from_cols_array(&[
            f / aspect, 0.0, 0.0, 0.0,
            0.0, f, 0.0, 0.0,
            0.0, 0.0, 0.0, -1.0,
            0.0, 0.0, znear, 0.0,
        ])
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub(crate) fn new() -> Self {
        Self {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
        }
    }

    pub(crate) fn update_view_proj(&mut self, camera: &CameraState) {
        self.view_proj = camera.build_view_projection_matrix().to_cols_array_2d();
    }
}
