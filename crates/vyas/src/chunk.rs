use std::collections::HashMap;

use glam::Vec3;

use crate::{
    camera::CameraState,
    config::RenderConfig,
    ecs::{Entity, World},
    frustum::Frustum,
    mesh::Mesh,
    position::GridPosition,
    voxel::Voxel,
};

pub(crate) struct Chunk {
    pub(crate) position: GridPosition,
    pub(crate) voxels: HashMap<GridPosition, Voxel>,
    pub(crate) dirty: bool,
    _mesh: Option<Mesh>,
}

impl Chunk {
    pub(crate) fn new(position: GridPosition) -> Self {
        Self {
            position,
            voxels: HashMap::new(),
            dirty: true,
            _mesh: None,
        }
    }

    pub(crate) fn mesh(&mut self, render_config: &RenderConfig) -> &Mesh {
        let dirty = self.dirty;
        self.dirty = false;

        if dirty {
            self._mesh.insert(Mesh::generate_mesh(
                &self.position,
                &self.voxels,
                render_config,
            ))
        } else {
            self._mesh.get_or_insert_with(|| {
                Mesh::generate_mesh(&self.position, &self.voxels, render_config)
            })
        }
    }

    pub(crate) fn set_voxel(&mut self, position: GridPosition, voxel: Voxel) {
        self.voxels.insert(position, voxel);
        self.dirty = true;
    }

    pub(crate) fn contains(&self, position: &GridPosition) -> bool {
        self.voxels.contains_key(position)
    }
}

pub(crate) struct ChunkMap {
    pub(crate) map: HashMap<GridPosition, Entity>,
}

impl ChunkMap {
    pub(crate) fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub(crate) fn get(&self, position: &GridPosition) -> Option<&Entity> {
        self.map.get(position)
    }

    pub(crate) fn insert(&mut self, position: GridPosition, entity: Entity) {
        self.map.insert(position, entity);
    }

    pub(crate) fn visible_chunk_entities(&self, world: &World) -> Vec<Entity> {
        let camera = world.resource::<CameraState>();
        let render_config = *world.resource::<RenderConfig>();

        let camera_chunk_position = camera.position.to_chunk_position(&render_config);
        let frustum = Frustum::from_view_projection(&camera.build_view_projection_matrix());
        let mut chunks = Vec::with_capacity(self.map.len());

        for (chunk_position, entity) in &self.map {
            if !Self::chunk_in_render_distance(
                chunk_position,
                &camera_chunk_position,
                render_config.max_render_distance,
            ) {
                continue;
            }

            if !Self::chunk_intersects_frustum(chunk_position, &frustum, &render_config) {
                continue;
            }

            chunks.push(*entity);
        }

        chunks
    }

    fn chunk_in_render_distance(
        chunk_position: &GridPosition,
        camera_chunk_position: &GridPosition,
        max_render_distance: i32,
    ) -> bool {
        (chunk_position.x - camera_chunk_position.x).abs() <= max_render_distance
            && (chunk_position.y - camera_chunk_position.y).abs() <= max_render_distance
            && (chunk_position.z - camera_chunk_position.z).abs() <= max_render_distance
    }

    fn chunk_intersects_frustum(
        chunk_position: &GridPosition,
        frustum: &Frustum,
        render_config: &RenderConfig,
    ) -> bool {
        let min = Vec3::from(&chunk_position.to_world_position(render_config))
            - Vec3::splat(render_config.voxel_size * 0.5);
        let max = min + Vec3::splat(render_config.chunk_size as f32 * render_config.voxel_size);

        frustum.intersects_aabb(min, max)
    }
}
