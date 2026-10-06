use std::collections::HashMap;

use crate::{
    color::ColorPaletteState, config::BRICKMAP_SIZE, position::GridPosition, voxel::Voxel,
};

#[derive(Debug, Clone)]
pub(crate) struct Brickmap {
    pub(crate) position: GridPosition,
    pub(crate) voxels: HashMap<GridPosition, Voxel>,
    pub(crate) gpu: BrickmapGpuData,
}

impl Brickmap {
    pub(crate) fn new(position: GridPosition) -> Self {
        Self {
            position,
            voxels: HashMap::new(),
            gpu: BrickmapGpuData::default(),
        }
    }

    pub(crate) fn set_voxel(
        &mut self,
        position: GridPosition,
        voxel: Voxel,
        color_palette: &ColorPaletteState,
    ) {
        let GridPosition { x, y, z } = position;
        let prev = self.voxels.insert(position, voxel);

        self.update_gpu_voxel(GridPosition { x, y, z }, color_palette);

        let is_new = prev.is_none();

        if !is_new {
            return;
        }

        for neighbor in [
            GridPosition { x: x + 1, y, z },
            GridPosition { x: x - 1, y, z },
            GridPosition { x, y: y + 1, z },
            GridPosition { x, y: y - 1, z },
            GridPosition { x, y, z: z + 1 },
            GridPosition { x, y, z: z - 1 },
        ] {
            if self.voxels.contains_key(&neighbor) {
                self.update_gpu_voxel(neighbor, color_palette);
            }
        }
    }

    fn update_gpu_voxel(&mut self, position: GridPosition, color_palette: &ColorPaletteState) {
        let GridPosition { x, y, z } = position;

        let index = position.to_index(BRICKMAP_SIZE as i32);

        let word_index = index / 32;
        let bit = 1u32 << (index % 32);
        let solid_mask = &mut self.gpu.brickmap.solid_mask;

        let color_index = solid_mask[..word_index]
            .iter()
            .map(|word| word.count_ones() as usize)
            .sum::<usize>()
            + (solid_mask[word_index] & (bit - 1)).count_ones() as usize;

        let is_prev_hidden = solid_mask[word_index] & bit == 0;

        let is_hidden = self.voxels.contains_key(&GridPosition { x: x + 1, y, z })
            && self.voxels.contains_key(&GridPosition { x: x - 1, y, z })
            && self.voxels.contains_key(&GridPosition { x, y: y + 1, z })
            && self.voxels.contains_key(&GridPosition { x, y: y - 1, z })
            && self.voxels.contains_key(&GridPosition { x, y, z: z + 1 })
            && self.voxels.contains_key(&GridPosition { x, y, z: z - 1 });

        if is_hidden && is_prev_hidden {
            return;
        }

        if is_hidden && !is_prev_hidden {
            solid_mask[word_index] &= !bit;
            self.gpu.colors.remove(color_index);
            return;
        }

        let Some(voxel) = self.voxels.get(&position) else {
            return;
        };

        let Some(color) = color_palette.get(&voxel.palette_color) else {
            log::warn!(
                "{} does not exist in color palette",
                voxel.palette_color.name()
            );
            return;
        };

        if is_prev_hidden {
            solid_mask[word_index] |= bit;
            self.gpu.colors.insert(color_index, color.index);
        } else {
            self.gpu.colors[color_index] = color.index;
        }
    }

    pub(crate) fn contains(&self, position: &GridPosition) -> bool {
        self.voxels.contains_key(position)
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.voxels.is_empty()
    }

    pub(crate) fn to_gpu(&self) -> BrickmapGpuData {
        let mut colors = self.gpu.colors.clone();
        let padded_len = colors.len().div_ceil(4) * 4;
        colors.resize(padded_len, 0);

        BrickmapGpuData {
            brickmap: self.gpu.brickmap,
            colors,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub(crate) struct BrickmapGpuData {
    pub(crate) brickmap: BrickmapGpu,
    pub(crate) colors: Vec<u8>,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct BrickmapGpu {
    pub(crate) solid_mask: [u32; 16],
    pub(crate) color_index: u32,
}

impl BrickmapGpu {
    pub(crate) fn is_empty(&self) -> bool {
        self.solid_mask.iter().all(|i| *i == 0)
    }
}
