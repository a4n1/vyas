#[derive(Clone, Copy, Debug)]
pub struct RenderConfig {
    pub chunk_size: u32,
    pub voxel_size: f32,
    pub max_render_distance: i32,
    pub max_buffer_size: u64,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            chunk_size: 16,
            voxel_size: 1.0,
            max_render_distance: 128,
            max_buffer_size: 268435456,
        }
    }
}
