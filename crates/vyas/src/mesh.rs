use std::collections::HashMap;

use crate::{config::RenderConfig, position::GridPosition, vertex::Vertex, voxel::Voxel};

#[derive(Clone)]
pub(crate) struct Mesh {
    pub(crate) vertices: Vec<Vertex>,
    pub(crate) indices: Vec<u32>,
}

impl Mesh {
    pub(crate) fn generate_mesh(
        chunk_position: &GridPosition,
        voxels: &HashMap<GridPosition, Voxel>,
        render_config: &RenderConfig,
    ) -> Mesh {
        let mut vertices: Vec<Vertex> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();

        for (index, (local_position, voxel)) in voxels.iter().enumerate() {
            let (v, i) = build_voxel(
                chunk_position,
                local_position,
                voxel,
                (index as u32) * 8,
                render_config,
            );

            vertices.extend(v);
            indices.extend(i);
        }

        Mesh { vertices, indices }
    }
}

fn build_voxel(
    chunk_position: &GridPosition,
    local_position: &GridPosition,
    voxel: &Voxel,
    indices_index: u32,
    render_config: &RenderConfig,
) -> (Vec<Vertex>, Vec<u32>) {
    let color = [voxel.color.r(), voxel.color.g(), voxel.color.b()];

    let half_size = render_config.voxel_size * 0.5;

    let x = (chunk_position.x * render_config.chunk_size as i32 + local_position.x) as f32
        * render_config.voxel_size;
    let y = (chunk_position.y * render_config.chunk_size as i32 + local_position.y) as f32
        * render_config.voxel_size;
    let z = (chunk_position.z * render_config.chunk_size as i32 + local_position.z) as f32
        * render_config.voxel_size;

    let vertices = vec![
        // Front Face
        Vertex {
            position: [x - half_size, y - half_size, z + half_size],
            color,
        },
        Vertex {
            position: [x + half_size, y - half_size, z + half_size],
            color,
        },
        Vertex {
            position: [x + half_size, y + half_size, z + half_size],
            color,
        },
        Vertex {
            position: [x - half_size, y + half_size, z + half_size],
            color,
        },
        // Rear Face
        Vertex {
            position: [x - half_size, y - half_size, z - half_size],
            color,
        },
        Vertex {
            position: [x + half_size, y - half_size, z - half_size],
            color,
        },
        Vertex {
            position: [x + half_size, y + half_size, z - half_size],
            color,
        },
        Vertex {
            position: [x - half_size, y + half_size, z - half_size],
            color,
        },
    ];

    #[allow(clippy::identity_op)]
    let indices = vec![
        // Front
        indices_index + 0,
        indices_index + 1,
        indices_index + 2,
        indices_index + 2,
        indices_index + 3,
        indices_index + 0,
        // Back
        indices_index + 5,
        indices_index + 4,
        indices_index + 7,
        indices_index + 7,
        indices_index + 6,
        indices_index + 5,
        // Left
        indices_index + 4,
        indices_index + 0,
        indices_index + 3,
        indices_index + 3,
        indices_index + 7,
        indices_index + 4,
        // Right
        indices_index + 1,
        indices_index + 5,
        indices_index + 6,
        indices_index + 6,
        indices_index + 2,
        indices_index + 1,
        // Top
        indices_index + 3,
        indices_index + 2,
        indices_index + 6,
        indices_index + 6,
        indices_index + 7,
        indices_index + 3,
        // Bottom
        indices_index + 4,
        indices_index + 5,
        indices_index + 1,
        indices_index + 1,
        indices_index + 0,
        indices_index + 4,
    ];

    (vertices, indices)
}
