use std::collections::HashMap;

use crate::{
    color::ColorPaletteState,
    config::{BRICKMAP_SIZE, FALLBACK_BRICKMAP_SIZE},
    position::GridPosition,
    voxel::Voxel,
};

#[derive(Debug)]
pub(crate) struct Brickmap {
    pub(crate) position: GridPosition,
    pub(crate) voxels: HashMap<GridPosition, Voxel>,
    pub(crate) gpu: BrickmapGpuData,
    pub(crate) fallback_gpu: FallbackBrickmapGpu,
}

impl Brickmap {
    pub(crate) fn new(position: GridPosition) -> Self {
        Self {
            position,
            voxels: HashMap::new(),
            gpu: BrickmapGpuData::default(),
            fallback_gpu: FallbackBrickmapGpu::default(),
        }
    }

    pub(crate) fn set_voxel(&mut self, position: GridPosition, voxel: Voxel) {
        self.voxels.insert(position, voxel);
    }

    pub(crate) fn rebuild(&mut self, color_palette: &ColorPaletteState) {
        for position in self.voxels.clone().keys() {
            self.update_gpu_voxel(position, color_palette);
        }
    }

    fn update_gpu_voxel(&mut self, position: &GridPosition, color_palette: &ColorPaletteState) {
        let GridPosition { x, y, z } = position.clone();

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

        let Some(voxel) = self.voxels.get(position) else {
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

    pub(crate) fn rebuild_fallback(&mut self, color_palette: &ColorPaletteState) {
        let mut solid_mask: u32 = 0;
        let mut colors: [u32; 2] = [0; 2];

        let fallback_brickmap_size = FALLBACK_BRICKMAP_SIZE as i32;

        for x in 0..fallback_brickmap_size {
            for y in 0..fallback_brickmap_size {
                for z in 0..fallback_brickmap_size {
                    let position = GridPosition { x, y, z };
                    let Some(voxels) = self.get_fallback_chunk(&position) else {
                        continue;
                    };

                    let idx = position.to_index(fallback_brickmap_size);
                    solid_mask |= 1u32 << idx;

                    let word_index = idx / 4;
                    let bit_index = idx % 4 * 8;
                    let color = Self::get_fallback_chunk_color(&voxels, color_palette);
                    colors[word_index] |= (color as u32) << bit_index;
                }
            }
        }

        self.fallback_gpu.solid_mask = solid_mask;
        self.fallback_gpu.colors = colors;
    }

    fn get_fallback_chunk(&self, position: &GridPosition) -> Option<Vec<Voxel>> {
        let mut voxels: Vec<Voxel> = Vec::new();

        let range = (BRICKMAP_SIZE / FALLBACK_BRICKMAP_SIZE) as i32;

        for x in 0..range {
            for y in 0..range {
                for z in 0..range {
                    let position = &GridPosition {
                        x: x + position.x * range,
                        y: y + position.y * range,
                        z: z + position.z * range,
                    };

                    if let Some(voxel) = self.voxels.get(position) {
                        voxels.push(voxel.clone());
                    }
                }
            }
        }

        if voxels.is_empty() {
            None
        } else {
            Some(voxels)
        }
    }

    fn get_fallback_chunk_color(voxels: &[Voxel], color_palette: &ColorPaletteState) -> u8 {
        let mut counts: HashMap<u8, u8> = HashMap::new();

        for voxel in voxels {
            let Some(color) = color_palette.get(&voxel.palette_color) else {
                log::warn!(
                    "{} does not exist in color palette",
                    voxel.palette_color.name()
                );
                continue;
            };

            *counts.entry(color.index).or_default() += 1;
        }

        counts
            .into_iter()
            .max_by_key(|&(_, count)| count)
            .map(|(num, _)| num)
            .unwrap_or_default()
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

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct FallbackBrickmapGpu {
    pub(crate) solid_mask: u32,
    pub(crate) colors: [u32; 2],
    padding: u32,
}
