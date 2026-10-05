use std::collections::HashMap;

use crate::{
    color::ColorPaletteState, config::BRICKMAP_SIZE, position::GridPosition, voxel::Voxel,
};

#[derive(Debug, Clone)]
pub(crate) struct Brickmap {
    pub(crate) position: GridPosition,
    pub(crate) voxels: HashMap<GridPosition, Voxel>,
}

impl Brickmap {
    pub(crate) fn new(position: GridPosition) -> Self {
        Self {
            position,
            voxels: HashMap::new(),
        }
    }

    pub(crate) fn set_voxel(&mut self, position: GridPosition, voxel: Voxel) {
        self.voxels.insert(position, voxel);
    }

    pub(crate) fn contains(&self, position: &GridPosition) -> bool {
        self.voxels.contains_key(position)
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.voxels.is_empty()
    }

    pub(crate) fn to_gpu(&self, color_palette: &ColorPaletteState) -> BrickmapGpuData {
        let mut solid_mask = [0u32; 16];
        let mut colors: Vec<u8> = Vec::with_capacity(BRICKMAP_SIZE.pow(3) as usize);

        for z in 0..BRICKMAP_SIZE {
            for y in 0..BRICKMAP_SIZE {
                for x in 0..BRICKMAP_SIZE {
                    let x = x as i32;
                    let y = y as i32;
                    let z = z as i32;

                    if self.voxels.contains_key(&GridPosition { x: x + 1, y, z })
                        && self.voxels.contains_key(&GridPosition { x: x - 1, y, z })
                        && self.voxels.contains_key(&GridPosition { x, y: y + 1, z })
                        && self.voxels.contains_key(&GridPosition { x, y: y - 1, z })
                        && self.voxels.contains_key(&GridPosition { x, y, z: z + 1 })
                        && self.voxels.contains_key(&GridPosition { x, y, z: z - 1 })
                    {
                        continue;
                    }

                    if let Some(voxel) = self.voxels.get(&GridPosition { x, y, z }) {
                        let Some(color) = color_palette.get(&voxel.palette_color) else {
                            log::warn!(
                                "{} does not exist in color palette",
                                voxel.palette_color.name()
                            );
                            continue;
                        };

                        let index = GridPosition { x, y, z }.to_index(BRICKMAP_SIZE as i32);

                        let word_index = index / 32;
                        let bit_index = index % 32;

                        solid_mask[word_index] |= 1u32 << bit_index;

                        colors.push(color.index);
                    }
                }
            }
        }

        let brickmap = BrickmapGpu {
            solid_mask,
            color_index: 0,
        };

        BrickmapGpuData { brickmap, colors }
    }
}

pub(crate) struct BrickmapGpuData {
    pub(crate) brickmap: BrickmapGpu,
    pub(crate) colors: Vec<u8>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct BrickmapGpu {
    pub(crate) solid_mask: [u32; 16],
    pub(crate) color_index: u32,
}

impl BrickmapGpu {
    pub(crate) fn is_empty(&self) -> bool {
        self.solid_mask.iter().all(|i| *i == 0)
    }
}
