use glam::{BVec3A, IVec3, Vec3A, Vec3Swizzles};

use crate::{
    chunk::{Chunk, ChunkMap},
    config::RenderConfig,
    ecs::World,
    position::{GridPosition, VoxelFace},
};

#[derive(Debug, Clone)]
pub struct VoxelHit {
    pub position: GridPosition,
    pub face: VoxelFace,
}

pub(crate) struct Ray {
    pub(crate) origin: Vec3A,
    pub(crate) direction: Vec3A,
}

#[allow(clippy::upper_case_acronyms)]
pub(crate) struct DDA;

impl DDA {
    pub(crate) fn hit(ray: &Ray, world: &World) -> Option<VoxelHit> {
        let render_config = *world.resource::<RenderConfig>();
        let max_distance = render_config.max_render_distance as f32
            * render_config.chunk_size as f32
            * render_config.voxel_size;
        let grid_origin = ray.origin / render_config.voxel_size + Vec3A::splat(0.5);

        let mut dda = DDAState::from_pos_and_dir(grid_origin, ray.direction);

        loop {
            let position = GridPosition {
                x: dda.next_voxel_pos.x,
                y: dda.next_voxel_pos.y,
                z: dda.next_voxel_pos.z,
            };

            if Self::contains_voxel(&position, world, &render_config)
                && let Some(face) = VoxelFace::from_normal(dda.hit_normal())
            {
                return Some(VoxelHit { position, face });
            }

            dda.step_mut();

            if dda.hit_distance() * render_config.voxel_size > max_distance {
                return None;
            }
        }
    }

    fn contains_voxel(
        position: &GridPosition,
        world: &World,
        render_config: &RenderConfig,
    ) -> bool {
        let chunk_position = position.to_chunk_position(render_config);
        let local_position = position.to_local_position(render_config);

        let chunk_map = world.resource::<ChunkMap>();
        let Some(entity) = chunk_map.get(&chunk_position) else {
            return false;
        };

        world
            .get::<Chunk>(*entity)
            .is_some_and(|chunk| chunk.contains(&local_position))
    }
}

#[derive(Debug, Clone, Copy)]
struct DDAState {
    /// The current largest component of/to the next boundary.
    max_boundary_mask: BVec3A,

    /// Per-component signum to the next voxel position.
    diff_voxel_pos: IVec3,

    /// The next voxel position to be visited.
    next_voxel_pos: IVec3,

    /// Per-component distance to next boundary plane.
    diff_boundary: Vec3A,

    /// The distances to the next voxel boundary on the X/Y/Z axes.
    next_boundary: Vec3A,
}

impl Iterator for DDAState {
    type Item = DDAState;

    fn next(&mut self) -> Option<Self::Item> {
        self.step_mut();
        Some(*self)
    }
}

impl DDAState {
    fn from_pos_and_dir(ray_origin: Vec3A, ray_direction: Vec3A) -> Self {
        let ray_origin_grid = ray_origin.floor();
        let ray_origin_grid_i = ray_origin_grid.as_ivec3();

        let mut ray_direction = ray_direction;
        if ray_direction.x == 0.0 {
            ray_direction.x = 0.00001;
        }
        if ray_direction.y == 0.0 {
            ray_direction.y = 0.00001;
        }
        if ray_direction.z == 0.0 {
            ray_direction.z = 0.00001;
        }
        let ray_direction = ray_direction.normalize();

        let ray_sign = ray_direction.signum();
        let ray_dir_inv = Vec3A::ONE / ray_direction;

        let ray_dist = ray_dir_inv.abs();
        let next_dist =
            (ray_sign * (ray_origin_grid - ray_origin) + (ray_sign * 0.5) + 0.5) * ray_dist;

        Self {
            diff_boundary: ray_dist,
            diff_voxel_pos: ray_sign.as_ivec3(),
            max_boundary_mask: BVec3A::FALSE,
            next_boundary: next_dist,
            next_voxel_pos: ray_origin_grid_i,
        }
    }

    fn step_mut(&mut self) {
        self.max_boundary_mask = self
            .next_boundary
            .xyz()
            .cmple(self.next_boundary.yzx().min(self.next_boundary.zxy()));
        self.next_voxel_pos += self.diff_voxel_pos * IVec3::from(self.max_boundary_mask);
        self.next_boundary += self.diff_boundary * Vec3A::from(self.max_boundary_mask);
    }

    fn hit_distance(&self) -> f32 {
        ((self.next_boundary - self.diff_boundary) * Vec3A::from(self.max_boundary_mask))
            .element_sum()
    }

    /// Normal of the hit on the next voxels boundary.
    fn hit_normal(&self) -> Vec3A {
        -self.diff_voxel_pos.as_vec3a() * Vec3A::from(self.max_boundary_mask)
    }
}
