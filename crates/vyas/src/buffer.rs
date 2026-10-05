use std::ops::Range;

use lru::LruCache;
use wgpu::{Limits, Queue};

use crate::{
    brickgrid::{BrickgridCell, BrickgridCellData},
    brickmap::{Brickmap, BrickmapGpu},
    color::ColorPaletteState,
    pipeline::Pipeline,
    prelude::GridPosition,
};

pub(crate) struct BrickmapBuffer {
    brickmaps: BufferState,
    colors: BufferState,
}

impl BrickmapBuffer {
    pub(crate) fn new() -> Self {
        let capacity = Limits::default().max_storage_buffer_binding_size;

        Self {
            brickmaps: BufferState::new(capacity),
            colors: BufferState::new(capacity),
        }
    }

    pub(crate) fn insert(
        &mut self,
        brickmap: &Brickmap,
        color_palette: &ColorPaletteState,
        pipeline: &Pipeline,
        queue: &Queue,
    ) -> Option<BrickgridCell> {
        let brickmap_gpu_data = brickmap.to_gpu(color_palette);

        let mut color_data = brickmap_gpu_data.colors;
        let padded_len = color_data.len().div_ceil(4) * 4;
        color_data.resize(padded_len, 0);
        let color_allocation = self
            .colors
            .alloc(color_data.len() as u64, &brickmap.position);

        let mut brickmap_gpu = brickmap_gpu_data.brickmap;
        let brickmap_allocation = self
            .brickmaps
            .alloc(size_of::<BrickmapGpu>() as u64, &brickmap.position);

        let color_index = color_allocation.offset;
        brickmap_gpu.color_index = color_index as u32;

        let brickmap_data = [brickmap_gpu];

        queue.write_buffer(&pipeline.color_buffer, color_allocation.offset, &color_data);

        queue.write_buffer(
            &pipeline.brickmap_buffer,
            brickmap_allocation.offset,
            bytemuck::cast_slice(&brickmap_data),
        );

        if brickmap_gpu.is_empty() {
            None
        } else {
            let index = (brickmap_allocation.offset / size_of::<BrickmapGpu>() as u64 + 1) as u32;

            Some(
                BrickgridCellData::Loaded {
                    brickmap_index: index,
                }
                .into(),
            )
        }
    }
}

struct BufferState {
    cache: LruCache<GridPosition, Allocation>,
    allocator: BufferAllocator,
}

impl BufferState {
    fn new(capacity: u64) -> Self {
        Self {
            cache: LruCache::unbounded(),
            allocator: BufferAllocator::new(capacity),
        }
    }

    fn alloc(&mut self, size: u64, position: &GridPosition) -> &mut Allocation {
        if !self.cache.contains(position) {
            let allocation = loop {
                if let Some(allocation) = self.allocator.alloc(size) {
                    break allocation;
                }

                let Some((_, evicted)) = self.cache.pop_lru() else {
                    panic!("failed to allocate memory, allocation exceeds buffer capacity");
                };

                self.allocator.free(&evicted);
            };

            self.cache.put(position.clone(), allocation);
        }

        self.cache
            .get_mut(position)
            .expect("failed to get allocation item")
    }
}

pub(crate) struct BufferAllocator {
    free_ranges: Vec<Range<u64>>,
}

#[derive(Default, Clone)]
pub(crate) struct Allocation {
    pub(crate) offset: u64,
    pub(crate) capacity: u64,
}

impl BufferAllocator {
    fn new(size: u64) -> Self {
        #[allow(clippy::single_range_in_vec_init)]
        Self {
            free_ranges: vec![0..size],
        }
    }

    fn alloc(&mut self, byte_len: u64) -> Option<Allocation> {
        if byte_len == 0 {
            return None;
        }

        for (idx, range) in self.free_ranges.iter().enumerate() {
            if range.end - range.start >= byte_len {
                let allocation = Allocation {
                    offset: range.start,
                    capacity: byte_len,
                };

                let new_range = (range.start + byte_len)..range.end;

                if new_range.start == range.end {
                    self.free_ranges.remove(idx);
                } else {
                    self.free_ranges[idx] = new_range;
                }

                return Some(allocation);
            }
        }

        None
    }

    fn free(&mut self, allocation: &Allocation) {
        if allocation.capacity == 0 {
            return;
        }

        let new_range = allocation.offset..(allocation.offset + allocation.capacity);

        let mut idx = 0;

        while idx < self.free_ranges.len() && self.free_ranges[idx].start < new_range.start {
            idx += 1;
        }

        self.free_ranges.insert(idx, new_range);

        if idx > 0 && self.free_ranges[idx - 1].end == self.free_ranges[idx].start {
            self.free_ranges[idx - 1].end = self.free_ranges[idx].end;
            self.free_ranges.remove(idx);
            idx -= 1;
        }

        if idx + 1 < self.free_ranges.len()
            && self.free_ranges[idx].end == self.free_ranges[idx + 1].start
        {
            self.free_ranges[idx].end = self.free_ranges[idx + 1].end;
            self.free_ranges.remove(idx + 1);
        }
    }
}
