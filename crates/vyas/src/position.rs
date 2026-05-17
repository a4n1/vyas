use glam::Vec3;

use crate::pipeline::CHUNK_SIZE;

#[derive(Debug, Default, Clone, Eq, Hash, PartialEq)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl GridPosition {
    pub(crate) fn to_chunk_position(&self) -> Self {
        let size = CHUNK_SIZE as i32;

        Self {
            x: self.x.div_euclid(size),
            y: self.y.div_euclid(size),
            z: self.z.div_euclid(size),
        }
    }

    pub(crate) fn to_local_position(&self) -> Self {
        let size = CHUNK_SIZE as i32;

        Self {
            x: self.x.rem_euclid(size),
            y: self.y.rem_euclid(size),
            z: self.z.rem_euclid(size),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct WorldPosition {
    pub x: f32,
    pub y: f32,
    pub z: f32,
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
