use serde::{Deserialize, Serialize};

use crate::{
    brickgrid::Brickgrid,
    brickmap::Brickmap,
    color::PaletteColor,
    ecs::{CommandQueue, SystemParam, World},
    position::GridPosition,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Voxel {
    pub palette_color: PaletteColor,
}

pub struct VoxelCommands {
    queue: *const CommandQueue,
}

impl VoxelCommands {
    pub fn spawn(&mut self, voxel_position: GridPosition, voxel: Voxel) {
        let queue = unsafe { &*self.queue };
        queue.borrow_mut().push(Box::new(move |world| {
            let brickmap_position = voxel_position.to_brickmap_position();
            let local_position = voxel_position.to_local_position();

            let entity = {
                let brickgrid = world.resource::<Brickgrid>();
                brickgrid.get(&brickmap_position).copied()
            };

            if let Some(entity) = entity {
                let mut brickmap = world
                    .get_mut::<Brickmap>(entity)
                    .expect("failed to get brickmap entity");
                brickmap.set_voxel(local_position, voxel);
            } else {
                let mut brickmap = Brickmap::new(brickmap_position.clone());

                brickmap.set_voxel(local_position, voxel);
                let entity = world.spawn((brickmap,));

                let mut brickgrid = world.resource_mut::<Brickgrid>();
                brickgrid.insert(brickmap_position, entity);
            }
        }));
    }

    pub fn despawn(&mut self, voxel_position: GridPosition) {
        let queue = unsafe { &*self.queue };
        queue.borrow_mut().push(Box::new(move |world| {
            let brickmap_position = voxel_position.to_brickmap_position();
            let local_position = voxel_position.to_local_position();

            let entity = {
                let brickgrid = world.resource::<Brickgrid>();
                brickgrid.get(&brickmap_position).copied()
            };

            if let Some(entity) = entity {
                let mut brickmap = world
                    .get_mut::<Brickmap>(entity)
                    .expect("failed to get brickmap entity");
                brickmap.remove_voxel(&local_position);
            }
        }));
    }
}

impl SystemParam for VoxelCommands {
    fn get(_world: *const World, commands: *const CommandQueue) -> Self {
        Self { queue: commands }
    }
}
