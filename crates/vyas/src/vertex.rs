#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

pub const VERTICES: &[Vertex] = &[
    // Front face
    Vertex {
        position: [-0.5, -0.5, 0.5],
        color: [0.8, 0.1, 0.1],
    }, // 0
    Vertex {
        position: [0.5, -0.5, 0.5],
        color: [0.8, 0.1, 0.1],
    }, // 1
    Vertex {
        position: [0.5, 0.5, 0.5],
        color: [0.8, 0.1, 0.1],
    }, // 2
    Vertex {
        position: [-0.5, 0.5, 0.5],
        color: [0.8, 0.1, 0.1],
    }, // 3
    // Back face
    Vertex {
        position: [-0.5, -0.5, -0.5],
        color: [0.1, 0.8, 0.1],
    }, // 4
    Vertex {
        position: [0.5, -0.5, -0.5],
        color: [0.1, 0.8, 0.1],
    }, // 5
    Vertex {
        position: [0.5, 0.5, -0.5],
        color: [0.1, 0.8, 0.1],
    }, // 6
    Vertex {
        position: [-0.5, 0.5, -0.5],
        color: [0.1, 0.8, 0.1],
    }, // 7
];

pub const INDICES: &[u16] = &[
    // front
    0, 1, 2, 2, 3, 0, // back
    5, 4, 7, 7, 6, 5, // left
    4, 0, 3, 3, 7, 4, // right
    1, 5, 6, 6, 2, 1, // top
    3, 2, 6, 6, 7, 3, // bottom
    4, 5, 1, 1, 0, 4,
];
