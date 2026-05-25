use std::{collections::HashMap, ops::Range};

use wgpu::Queue;

use crate::{chunk::Chunk, config::RenderConfig, ecs::Entity, pipeline::Pipeline};

pub(crate) struct ChunkBuffer {
    data: HashMap<Entity, ChunkBufferItem>,
    vertex_allocator: BufferAllocator,
    index_allocator: BufferAllocator,
    frame: u64,
}

#[derive(Default, Clone)]
pub(crate) struct ChunkBufferItem {
    pub(crate) index_allocation: Allocation,
    pub(crate) vertex_allocation: Allocation,
    pub(crate) vertex_byte_len: u64,
    pub(crate) index_byte_len: u64,
    pub(crate) mesh_index_count: u32,
    frame: u64,
}

impl ChunkBuffer {
    pub(crate) fn new(render_config: &RenderConfig) -> Self {
        Self {
            data: HashMap::new(),
            vertex_allocator: BufferAllocator::new(render_config),
            index_allocator: BufferAllocator::new(render_config),
            frame: 0,
        }
    }

    pub(crate) fn insert(
        &mut self,
        entity: &Entity,
        chunk: &mut Chunk,
        pipeline: &Pipeline,
        queue: &Queue,
        render_config: &RenderConfig,
    ) -> Option<ChunkBufferItem> {
        let needs_upload = chunk.dirty || !self.data.contains_key(entity);

        if needs_upload {
            self.upload_chunk(entity, chunk, pipeline, queue, render_config);
        }

        let buffer_chunk = self.data.get_mut(entity)?;

        buffer_chunk.frame = self.frame;

        Some(buffer_chunk.clone())
    }

    pub(crate) fn cleanup(&mut self) {
        self.frame += 1;

        self.data.retain(|_, chunk| {
            if self.frame.saturating_sub(chunk.frame) > 300 {
                self.vertex_allocator.free(&chunk.vertex_allocation);
                self.index_allocator.free(&chunk.index_allocation);

                false
            } else {
                true
            }
        });
    }

    fn upload_chunk(
        &mut self,
        entity: &Entity,
        chunk: &mut Chunk,
        pipeline: &Pipeline,
        queue: &Queue,
        render_config: &RenderConfig,
    ) {
        let mesh = chunk.mesh(render_config);

        let vertex_bytes = bytemuck::cast_slice(&mesh.vertices);
        let index_bytes = bytemuck::cast_slice(&mesh.indices);
        let vertex_byte_len = vertex_bytes.len() as u64;
        let index_byte_len = index_bytes.len() as u64;

        if vertex_byte_len == 0 || index_byte_len == 0 {
            self.free_buffer_chunk(entity);
            return;
        }

        let buffer_chunk = self.data.entry(*entity).or_default();

        let needs_new_slot = buffer_chunk.vertex_allocation.capacity < vertex_byte_len
            || buffer_chunk.index_allocation.capacity < index_byte_len;

        if needs_new_slot {
            self.vertex_allocator.free(&buffer_chunk.vertex_allocation);
            self.index_allocator.free(&buffer_chunk.index_allocation);

            buffer_chunk.vertex_allocation = self
                .vertex_allocator
                .alloc(vertex_byte_len)
                .expect("vertex buffer allocator exhausted");

            buffer_chunk.index_allocation = self
                .index_allocator
                .alloc(index_byte_len)
                .expect("index buffer allocator exhausted");
        }

        buffer_chunk.vertex_byte_len = vertex_byte_len;
        buffer_chunk.index_byte_len = index_byte_len;
        buffer_chunk.mesh_index_count = mesh.indices.len() as u32;
        buffer_chunk.frame = self.frame;

        queue.write_buffer(
            &pipeline.vertex_buffer,
            buffer_chunk.vertex_allocation.offset,
            vertex_bytes,
        );

        queue.write_buffer(
            &pipeline.index_buffer,
            buffer_chunk.index_allocation.offset,
            index_bytes,
        );
    }

    fn free_buffer_chunk(&mut self, entity: &Entity) {
        if let Some(buffer_chunk) = self.data.remove(entity) {
            self.vertex_allocator.free(&buffer_chunk.vertex_allocation);
            self.index_allocator.free(&buffer_chunk.index_allocation);
        }
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
    fn new(render_config: &RenderConfig) -> Self {
        #[allow(clippy::single_range_in_vec_init)]
        Self {
            free_ranges: vec![0..render_config.max_buffer_size],
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
