use std::collections::HashMap;

use crate::{
    camera::CameraState,
    config::RenderConfig,
    ecs::{Entity, World},
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
        let mut chunks = Vec::new();

        for y in camera_chunk_position.y - render_config.max_render_distance
            ..=camera_chunk_position.y + render_config.max_render_distance
        {
            for z in camera_chunk_position.z - render_config.max_render_distance
                ..=camera_chunk_position.z + render_config.max_render_distance
            {
                for x in camera_chunk_position.x - render_config.max_render_distance
                    ..=camera_chunk_position.x + render_config.max_render_distance
                {
                    let chunk_position = GridPosition { x, y, z };

                    if let Some(entity) = self.get(&chunk_position).copied() {
                        chunks.push(entity);
                    }
                }
            }
        }

        chunks
    }
}
