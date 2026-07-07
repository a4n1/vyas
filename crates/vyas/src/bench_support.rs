use rand::prelude::*;
use std::{collections::HashMap, hint::black_box};

use crate::{
    color::{Color, Srgb},
    config::RenderConfig,
    mesh::Mesh,
    position::GridPosition,
    voxel::Voxel,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MeshStats {
    pub vertices: usize,
    pub indices: usize,
}

pub fn generate_mesh_stats(
    chunk_position: &GridPosition,
    voxels: &HashMap<GridPosition, Voxel>,
    render_config: &RenderConfig,
) -> MeshStats {
    let mesh = Mesh::generate_mesh(chunk_position, voxels, render_config);

    let stats = MeshStats {
        vertices: mesh.vertices.len(),
        indices: mesh.indices.len(),
    };

    black_box(mesh);
    stats
}

pub fn build_voxels(render_config: &RenderConfig) -> HashMap<GridPosition, Voxel> {
    let chunk_size = render_config.chunk_size as i32;
    let max_voxels = chunk_size.pow(3) as u32;
    let mut voxels = HashMap::with_capacity(max_voxels as usize);

    let mut rng = rand::rng();
    let colors = [
        Color::Srgb(Srgb { r: 255, g: 0, b: 0 }),
        Color::Srgb(Srgb { r: 0, g: 255, b: 0 }),
        Color::Srgb(Srgb { r: 0, g: 0, b: 255 }),
    ];

    for x in 0..chunk_size {
        for y in 0..chunk_size {
            for z in 0..chunk_size {
                let color = colors.choose(&mut rng).unwrap().clone();
                voxels.insert(GridPosition { x, y, z }, Voxel { color });
            }
        }
    }

    voxels
}
