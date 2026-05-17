use crate::{
    chunk::{Chunk, ChunkMap},
    color::Color,
    ecs::{CommandQueue, SystemParam, World},
    position::GridPosition,
};

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
            let chunk_position = voxel_position.to_chunk_position();
            let local_position = voxel_position.to_local_position();

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
}

impl SystemParam for VoxelCommands {
    fn get(_world: *const World, commands: *const CommandQueue) -> Self {
        Self { queue: commands }
    }
}
