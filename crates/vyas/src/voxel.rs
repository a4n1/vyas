use serde::{Deserialize, Serialize};

use crate::{
    asset::VoxelAsset,
    chunk::{Chunk, ChunkMap},
    color::Color,
    config::RenderConfig,
    ecs::{CommandQueue, SystemParam, World},
    position::GridPosition,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Voxel {
    pub color: Color,
}

pub struct VoxelCommands {
    queue: *const CommandQueue,
}

impl VoxelCommands {
    pub fn spawn(&mut self, voxel_position: GridPosition, voxel: Voxel) {
        let queue = unsafe { &*self.queue };
        queue.borrow_mut().push(Box::new(move |world| {
            let render_config = *world.resource::<RenderConfig>();
            let chunk_position = voxel_position.to_chunk_position(&render_config);
            let local_position = voxel_position.to_local_position(&render_config);

            let entity = {
                let chunk_map = world.resource::<ChunkMap>();
                chunk_map.get(&chunk_position).copied()
            };

            if let Some(entity) = entity {
                let mut chunk = world
                    .get_mut::<Chunk>(entity)
                    .expect("failed to get chunk entity");
                chunk.set_voxel(local_position, voxel);
            } else {
                let mut chunk = Chunk::new(chunk_position.clone());

                chunk.set_voxel(local_position, voxel);
                let entity = world.spawn((chunk,));

                {
                    let mut chunk_map = world.resource_mut::<ChunkMap>();
                    chunk_map.insert(chunk_position, entity);
                }
            }
        }));
    }

    pub fn despawn(&mut self, voxel_position: GridPosition) {
        let queue = unsafe { &*self.queue };
        queue.borrow_mut().push(Box::new(move |world| {
            let render_config = *world.resource::<RenderConfig>();
            let chunk_position = voxel_position.to_chunk_position(&render_config);
            let local_position = voxel_position.to_local_position(&render_config);

            let entity = {
                let chunk_map = world.resource::<ChunkMap>();
                chunk_map.get(&chunk_position).copied()
            };

            if let Some(entity) = entity {
                let mut chunk = world
                    .get_mut::<Chunk>(entity)
                    .expect("failed to get chunk entity");
                chunk.remove_voxel(&local_position);
            }
        }));
    }

    pub fn spawn_asset(&mut self, asset: VoxelAsset, position: &GridPosition) {
        for (asset_position, voxel) in asset.grid {
            let x = asset_position.x + position.x;
            let y = asset_position.y + position.y;
            let z = asset_position.z + position.z;

            self.spawn(GridPosition { x, y, z }, voxel);
        }
    }
}

impl SystemParam for VoxelCommands {
    fn get(_world: *const World, commands: *const CommandQueue) -> Self {
        Self { queue: commands }
    }
}
