use std::collections::HashMap;

use glam::Vec3;

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

                    if !self.chunk_intersects_frustum(&chunk_position, &camera, &render_config) {
                        continue;
                    }

                    let Some(entity) = self.get(&chunk_position).copied() else {
                        continue;
                    };

                    chunks.push(entity);
                }
            }
        }

        chunks
    }

    fn chunk_intersects_frustum(
        &self,
        chunk_position: &GridPosition,
        camera: &CameraState,
        render_config: &RenderConfig,
    ) -> bool {
        let view_proj = camera.build_view_projection_matrix();

        let min = Vec3::from(&chunk_position.to_world_position(render_config))
            - Vec3::splat(render_config.voxel_size * 0.5);
        let max = min + Vec3::splat(render_config.chunk_size as f32 * render_config.voxel_size);

        let mut outside_left = true;
        let mut outside_right = true;
        let mut outside_top = true;
        let mut outside_bottom = true;
        let mut outside_near = true;

        let corners = [
            Vec3::new(min.x, min.y, min.z),
            Vec3::new(max.x, min.y, min.z),
            Vec3::new(min.x, max.y, min.z),
            Vec3::new(max.x, max.y, min.z),
            Vec3::new(min.x, min.y, max.z),
            Vec3::new(max.x, min.y, max.z),
            Vec3::new(min.x, max.y, max.z),
            Vec3::new(max.x, max.y, max.z),
        ];

        for corner in corners {
            let clip = view_proj * corner.extend(1.0);

            outside_left &= clip.x < -clip.w;
            outside_right &= clip.x > clip.w;
            outside_top &= clip.y > clip.w;
            outside_bottom &= clip.y < -clip.w;
            outside_near &= clip.z > clip.w;
        }

        !(outside_left || outside_right || outside_top || outside_bottom || outside_near)
    }
}
