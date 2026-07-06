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
        let mut indices_index = 0;

        for (local_position, voxel) in voxels {
            let (v, i) = build_voxel(
                chunk_position,
                local_position,
                voxel,
                &mut indices_index,
                voxels,
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
    indices_index: &mut u32,
    voxels: &HashMap<GridPosition, Voxel>,
    render_config: &RenderConfig,
) -> (Vec<Vertex>, Vec<u32>) {
    let color = voxel.color.linear_rgb();

    let half_size = render_config.voxel_size * 0.5;

    let x = (chunk_position.x * render_config.chunk_size as i32 + local_position.x) as f32
        * render_config.voxel_size;
    let y = (chunk_position.y * render_config.chunk_size as i32 + local_position.y) as f32
        * render_config.voxel_size;
    let z = (chunk_position.z * render_config.chunk_size as i32 + local_position.z) as f32
        * render_config.voxel_size;

    let mut vertices = Vec::with_capacity(24);
    let mut indices = Vec::with_capacity(36);

    for direction in [
        Direction::Front,
        Direction::Back,
        Direction::Left,
        Direction::Right,
        Direction::Top,
        Direction::Bottom,
    ] {
        let is_adjacent_face = match direction {
            Direction::Left => voxels.contains_key(&GridPosition {
                x: local_position.x - 1,
                y: local_position.y,
                z: local_position.z,
            }),
            Direction::Right => voxels.contains_key(&GridPosition {
                x: local_position.x + 1,
                y: local_position.y,
                z: local_position.z,
            }),
            Direction::Top => voxels.contains_key(&GridPosition {
                x: local_position.x,
                y: local_position.y + 1,
                z: local_position.z,
            }),
            Direction::Bottom => voxels.contains_key(&GridPosition {
                x: local_position.x,
                y: local_position.y - 1,
                z: local_position.z,
            }),
            Direction::Front => voxels.contains_key(&GridPosition {
                x: local_position.x,
                y: local_position.y,
                z: local_position.z + 1,
            }),
            Direction::Back => voxels.contains_key(&GridPosition {
                x: local_position.x,
                y: local_position.y,
                z: local_position.z - 1,
            }),
        };

        if is_adjacent_face {
            continue;
        }

        let (quad_vertices, quad_indices) =
            generate_quad(direction, x, y, z, half_size, *indices_index, color);

        *indices_index += quad_vertices.len() as u32;

        vertices.extend(quad_vertices);
        indices.extend(quad_indices);
    }

    (vertices, indices)
}

#[derive(Clone, Copy)]
enum Direction {
    Left,
    Right,
    Top,
    Bottom,
    Front,
    Back,
}

fn generate_quad(
    direction: Direction,
    x: f32,
    y: f32,
    z: f32,
    half_size: f32,
    indices_index: u32,
    color: [f32; 3],
) -> (Vec<Vertex>, Vec<u32>) {
    let corners = match direction {
        Direction::Front => [
            [x - half_size, y - half_size, z + half_size],
            [x + half_size, y - half_size, z + half_size],
            [x + half_size, y + half_size, z + half_size],
            [x - half_size, y + half_size, z + half_size],
        ],
        Direction::Back => [
            [x + half_size, y - half_size, z - half_size],
            [x - half_size, y - half_size, z - half_size],
            [x - half_size, y + half_size, z - half_size],
            [x + half_size, y + half_size, z - half_size],
        ],
        Direction::Left => [
            [x - half_size, y - half_size, z - half_size],
            [x - half_size, y - half_size, z + half_size],
            [x - half_size, y + half_size, z + half_size],
            [x - half_size, y + half_size, z - half_size],
        ],
        Direction::Right => [
            [x + half_size, y - half_size, z + half_size],
            [x + half_size, y - half_size, z - half_size],
            [x + half_size, y + half_size, z - half_size],
            [x + half_size, y + half_size, z + half_size],
        ],
        Direction::Top => [
            [x - half_size, y + half_size, z + half_size],
            [x + half_size, y + half_size, z + half_size],
            [x + half_size, y + half_size, z - half_size],
            [x - half_size, y + half_size, z - half_size],
        ],
        Direction::Bottom => [
            [x - half_size, y - half_size, z - half_size],
            [x + half_size, y - half_size, z - half_size],
            [x + half_size, y - half_size, z + half_size],
            [x - half_size, y - half_size, z + half_size],
        ],
    };

    let vertices = corners
        .into_iter()
        .map(|position| Vertex { position, color })
        .collect();

    #[allow(clippy::identity_op)]
    let indices = vec![
        indices_index + 0,
        indices_index + 1,
        indices_index + 2,
        indices_index + 2,
        indices_index + 3,
        indices_index + 0,
    ];

    (vertices, indices)
}
