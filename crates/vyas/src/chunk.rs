use std::collections::HashMap;

use crate::{ecs::Entity, mesh::Mesh, position::GridPosition, voxel::Voxel};

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

    pub(crate) fn mesh(&mut self) -> &Mesh {
        let dirty = self.dirty;
        self.dirty = false;

        if dirty {
            self._mesh
                .insert(Mesh::generate_mesh(&self.position, &self.voxels))
        } else {
            self._mesh
                .get_or_insert_with(|| Mesh::generate_mesh(&self.position, &self.voxels))
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
}
