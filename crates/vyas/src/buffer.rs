use std::ops::Range;

use lru::LruCache;
use wgpu::{Limits, Queue};

use crate::{
    brickgrid::{BrickgridCellData, Feedback},
    brickmap::{Brickmap, BrickmapGpu},
    color::ColorPaletteState,
    pipeline::Pipeline,
    prelude::GridPosition,
};

pub(crate) struct BrickmapBuffer {
    brickmaps: BufferState<BrickmapItem>,
    colors: BufferState<ColorItem>,
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
        brickmap: &mut Brickmap,
        brickgrid_index: usize,
        color_palette: &ColorPaletteState,
        feedback: &Feedback,
        pipeline: &Pipeline,
        queue: &Queue,
    ) -> BrickgridCellData {
        if brickmap.dirty {
            self.brickmaps.free(&brickmap.position);
            self.colors.free(&brickmap.position);
            brickmap.dirty = false;
        }

        if brickmap.is_empty() {
            return BrickgridCellData::Empty;
        }

        let needs_upload = feedback.is_requested(brickgrid_index as u32);

        if needs_upload {
            self.upload_brickmap(brickmap, color_palette, pipeline, queue);
        }

        if let Some(brickmap_item) = self.brickmaps.get(&brickmap.position) {
            brickmap_item.brickgrid_cell.clone()
        } else {
            BrickgridCellData::Unloaded
        }
    }

    fn upload_brickmap(
        &mut self,
        brickmap: &mut Brickmap,
        color_palette: &ColorPaletteState,
        pipeline: &Pipeline,
        queue: &Queue,
    ) {
        let brickmap_gpu_data = brickmap.to_gpu(color_palette);

        let mut color_data = brickmap_gpu_data.colors;
        let padded_len = color_data.len().div_ceil(4) * 4;
        color_data.resize(padded_len, 0);
        let color_item = self
            .colors
            .alloc(color_data.len() as u64, &brickmap.position);

        let mut brickmap_gpu = brickmap_gpu_data.brickmap;
        let brickmap_item = self
            .brickmaps
            .alloc(size_of::<BrickmapGpu>() as u64, &brickmap.position);

        let color_index = color_item.allocation.offset;
        brickmap_gpu.color_index = color_index as u32;

        let brickmap_data = [brickmap_gpu];

        queue.write_buffer(
            &pipeline.color_buffer,
            color_item.allocation.offset,
            &color_data,
        );

        queue.write_buffer(
            &pipeline.brickmap_buffer,
            brickmap_item.allocation.offset,
            bytemuck::cast_slice(&brickmap_data),
        );

        if !brickmap_gpu.is_empty() {
            let index =
                (brickmap_item.allocation.offset / size_of::<BrickmapGpu>() as u64 + 1) as u32;

            brickmap_item.brickgrid_cell = BrickgridCellData::Loaded {
                brickmap_index: index,
            };
        }

        brickmap.dirty = false;
    }
}

struct BufferState<V: AllocationItem> {
    cache: LruCache<GridPosition, V>,
    allocator: BufferAllocator,
}

impl<V: AllocationItem> BufferState<V> {
    fn new(capacity: u64) -> Self {
        Self {
            cache: LruCache::unbounded(),
            allocator: BufferAllocator::new(capacity),
        }
    }

    fn get(&mut self, position: &GridPosition) -> Option<&V> {
        self.cache.get(position)
    }

    fn free(&mut self, position: &GridPosition) {
        if let Some(value) = self.cache.pop(position) {
            self.allocator.free(value.allocation());
        }
    }

    fn alloc(&mut self, size: u64, position: &GridPosition) -> &mut V {
        if !self.cache.contains(position) {
            let allocation = loop {
                if let Some(allocation) = self.allocator.alloc(size) {
                    break allocation;
                }

                let Some((_, evicted)) = self.cache.pop_lru() else {
                    panic!("failed to allocate memory, allocation exceeds buffer capacity");
                };

                self.allocator.free(evicted.allocation());
            };

            self.cache.put(position.clone(), V::new(allocation));
        }

        self.cache
            .get_mut(position)
            .expect("failed to get allocation item")
    }
}

trait AllocationItem {
    fn new(allocation: Allocation) -> Self;
    fn allocation(&self) -> &Allocation;
}

#[derive(Default, Clone)]
pub(crate) struct BrickmapItem {
    pub(crate) allocation: Allocation,
    pub(crate) brickgrid_cell: BrickgridCellData,
}

impl AllocationItem for BrickmapItem {
    fn new(allocation: Allocation) -> Self {
        Self {
            allocation,
            brickgrid_cell: BrickgridCellData::default(),
        }
    }

    fn allocation(&self) -> &Allocation {
        &self.allocation
    }
}

pub(crate) struct ColorItem {
    pub(crate) allocation: Allocation,
}

impl AllocationItem for ColorItem {
    fn new(allocation: Allocation) -> Self {
        Self { allocation }
    }

    fn allocation(&self) -> &Allocation {
        &self.allocation
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
