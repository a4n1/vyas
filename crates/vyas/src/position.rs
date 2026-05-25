use glam::Vec3;

use crate::config::RenderConfig;

#[derive(Debug, Default, Clone, Eq, Hash, PartialEq)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl GridPosition {
    pub(crate) fn to_chunk_position(&self, render_config: &RenderConfig) -> Self {
        let size = render_config.chunk_size as i32;

        Self {
            x: self.x.div_euclid(size),
            y: self.y.div_euclid(size),
            z: self.z.div_euclid(size),
        }
    }

    pub(crate) fn to_local_position(&self, render_config: &RenderConfig) -> Self {
        let size = render_config.chunk_size as i32;

        Self {
            x: self.x.rem_euclid(size),
            y: self.y.rem_euclid(size),
            z: self.z.rem_euclid(size),
        }
    }

    pub(crate) fn to_world_position(&self, render_config: &RenderConfig) -> WorldPosition {
        let scale = render_config.chunk_size as f32 * render_config.voxel_size;

        WorldPosition {
            x: self.x as f32 * scale,
            y: self.y as f32 * scale,
            z: self.z as f32 * scale,
        }
    }
}

impl From<&GridPosition> for Vec3 {
    fn from(GridPosition { x, y, z }: &GridPosition) -> Self {
        Vec3 {
            x: *x as f32,
            y: *y as f32,
            z: *z as f32,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct WorldPosition {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl WorldPosition {
    pub(crate) fn to_chunk_position(&self, render_config: &RenderConfig) -> GridPosition {
        let size = render_config.chunk_size as f32;

        GridPosition {
            x: (self.x / render_config.voxel_size).div_euclid(size) as i32,
            y: (self.y / render_config.voxel_size).div_euclid(size) as i32,
            z: (self.z / render_config.voxel_size).div_euclid(size) as i32,
        }
    }
}

impl From<&WorldPosition> for Vec3 {
    fn from(WorldPosition { x, y, z }: &WorldPosition) -> Self {
        Vec3 {
            x: *x,
            y: *y,
            z: *z,
        }
    }
}
