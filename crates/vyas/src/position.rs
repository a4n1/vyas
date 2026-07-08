use crate::config::RenderConfig;
use glam::{Vec3, Vec3A};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
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

    pub fn adjacent(&self, face: VoxelFace) -> Self {
        match face {
            VoxelFace::Left => Self {
                x: self.x - 1,
                ..self.clone()
            },
            VoxelFace::Right => Self {
                x: self.x + 1,
                ..self.clone()
            },
            VoxelFace::Bottom => Self {
                y: self.y - 1,
                ..self.clone()
            },
            VoxelFace::Top => Self {
                y: self.y + 1,
                ..self.clone()
            },
            VoxelFace::Back => Self {
                z: self.z - 1,
                ..self.clone()
            },
            VoxelFace::Front => Self {
                z: self.z + 1,
                ..self.clone()
            },
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

impl From<&WorldPosition> for Vec3A {
    fn from(WorldPosition { x, y, z }: &WorldPosition) -> Self {
        Vec3A::new(*x, *y, *z)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoxelFace {
    Left,
    Right,
    Bottom,
    Top,
    Back,
    Front,
}

impl VoxelFace {
    pub(crate) fn from_normal(normal: Vec3A) -> Option<Self> {
        if normal.x < 0.0 {
            Some(Self::Left)
        } else if normal.x > 0.0 {
            Some(Self::Right)
        } else if normal.y < 0.0 {
            Some(Self::Bottom)
        } else if normal.y > 0.0 {
            Some(Self::Top)
        } else if normal.z < 0.0 {
            Some(Self::Back)
        } else if normal.z > 0.0 {
            Some(Self::Front)
        } else {
            None
        }
    }
}
