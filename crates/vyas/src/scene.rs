use std::collections::{HashMap, HashSet};

use crate::{
    asset::Grid,
    brickgrid::{Brickgrid, BrickgridCell, BrickgridCellData},
    brickmap::{Brickmap, FallbackBrickmapGpu},
    color::ColorPaletteState,
    config::RenderConfigState,
    ecs::{Bundle, Entity, World},
    prelude::GridPosition,
};

pub struct Scene {
    pub grid: Grid,
}

impl Bundle for Scene {
    fn insert_into(self, _entity: Entity, world: &mut World) {
        let mut scene = world.resource_mut::<SceneState>();
        let render_config = *world.resource::<RenderConfigState>();
        let palette = world.resource::<ColorPaletteState>();
        let mut brickmaps = HashSet::new();

        for block in self.grid {
            let brickmap_position = block.position.to_brickmap_position();
            let local_position = block.position.to_local_position();
            brickmaps.insert(brickmap_position.clone());

            let brickmap = scene
                .brickmaps
                .entry(brickmap_position.clone())
                .or_insert_with(|| Brickmap::new(brickmap_position));

            brickmap.set_voxel(local_position, block.voxel);
        }

        for position in brickmaps {
            if let Some(brickmap) = scene.brickmaps.get_mut(&position) {
                brickmap.rebuild(&palette);
                brickmap.rebuild_fallback(&palette);
            }
        }

        let size = render_config.brickgrid_size;
        scene.rebuild(size);
    }
}

#[derive(Debug)]
pub(crate) struct SceneState {
    brickgrid: Brickgrid,
    brickmaps: HashMap<GridPosition, Brickmap>,
    camera_position: GridPosition,
}

impl SceneState {
    pub(crate) fn new(brickgrid_size: u32, camera_position: GridPosition) -> Self {
        SceneState {
            brickgrid: Brickgrid::new(brickgrid_size),
            brickmaps: HashMap::new(),
            camera_position,
        }
    }

    pub(crate) fn to_brickgrid_gpu(&self) -> &[BrickgridCell] {
        &self.brickgrid.data
    }

    pub(crate) fn to_fallback_brickmaps_gpu(&self) -> &[FallbackBrickmapGpu] {
        &self.brickgrid.fallback_data
    }

    pub(crate) fn get_brickmap_by_index(&self, idx: u32) -> Option<&Brickmap> {
        let size = self.brickgrid.size();
        let idx = idx as usize;
        if size == 0 || idx >= self.brickgrid.data.len() {
            return None;
        }

        let plane = size * size;
        let range = (size as i32 - 1) / 2;
        let position = GridPosition {
            x: self.camera_position.x + (idx % size) as i32 - range,
            y: self.camera_position.y + range - ((idx / size) % size) as i32,
            z: self.camera_position.z + range - (idx / plane) as i32,
        };
        let brickmap = self.brickmaps.get(&position)?;

        Some(brickmap)
    }

    pub(crate) fn get_brickmap_by_position(&self, position: &GridPosition) -> Option<&Brickmap> {
        self.brickmaps.get(position)
    }

    pub(crate) fn update_brickgrid(&mut self, idx: u32, cell: BrickgridCell) {
        self.brickgrid.data[idx as usize] = cell;
    }

    pub(crate) fn update(&mut self, camera_position: &GridPosition, brickgrid_size: u32) {
        let diff = camera_position.clone() - self.camera_position.clone();
        self.camera_position = camera_position.clone();

        if brickgrid_size as usize != self.brickgrid.size() {
            self.rebuild(brickgrid_size);
            return;
        }

        if diff == GridPosition::default() {
            return;
        }

        let size = self.brickgrid.size();

        let plane = size * size;
        let shift_x = diff.x.rem_euclid(size as i32) as usize;
        let shift_y = diff.y.rem_euclid(size as i32) as usize;
        let shift_z = diff.z.rem_euclid(size as i32) as usize;

        if shift_z != 0 {
            self.brickgrid.data.rotate_right(shift_z * plane);
            self.brickgrid.fallback_data.rotate_right(shift_z * plane);
        }

        if shift_y != 0 {
            for slice in self.brickgrid.data.chunks_exact_mut(plane) {
                slice.rotate_right(shift_y * size);
            }
            for slice in self.brickgrid.fallback_data.chunks_exact_mut(plane) {
                slice.rotate_right(shift_y * size);
            }
        }

        if shift_x != 0 {
            for row in self.brickgrid.data.chunks_exact_mut(size) {
                row.rotate_left(shift_x);
            }
            for row in self.brickgrid.fallback_data.chunks_exact_mut(size) {
                row.rotate_left(shift_x);
            }
        }

        let x_range = if diff.x > 0 {
            size - diff.x as usize..size
        } else {
            0..diff.x.unsigned_abs() as usize
        };
        for z in 0..size {
            for y in 0..size {
                for x in x_range.clone() {
                    self.refresh_brickgrid_cell(x, y, z, camera_position);
                }
            }
        }

        let y_range = if diff.y > 0 {
            0..diff.y as usize
        } else {
            size - diff.y.unsigned_abs() as usize..size
        };
        for z in 0..size {
            for y in y_range.clone() {
                for x in 0..size {
                    self.refresh_brickgrid_cell(x, y, z, camera_position);
                }
            }
        }

        let z_range = if diff.z > 0 {
            0..diff.z as usize
        } else {
            size - diff.z.unsigned_abs() as usize..size
        };
        for z in z_range {
            for y in 0..size {
                for x in 0..size {
                    self.refresh_brickgrid_cell(x, y, z, camera_position);
                }
            }
        }

        self.camera_position = camera_position.clone();
    }

    fn refresh_brickgrid_cell(
        &mut self,
        x: usize,
        y: usize,
        z: usize,
        camera_position: &GridPosition,
    ) {
        let size = self.brickgrid.size();

        let range = (size as i32 - 1) / 2;
        let position = GridPosition {
            x: camera_position.x + x as i32 - range,
            y: camera_position.y + range - y as i32,
            z: camera_position.z + range - z as i32,
        };

        let brickmap = self.brickmaps.get(&position);

        let cell = match brickmap {
            Some(brickmap) if !brickmap.is_empty() => BrickgridCellData::Unloaded,
            _ => BrickgridCellData::Empty,
        };

        let fallback = match brickmap {
            Some(brickmap) => brickmap.fallback_gpu,
            None => FallbackBrickmapGpu::default(),
        };

        let idx = x + y * size + z * size.pow(2);
        self.brickgrid.data[idx] = cell.into();
        self.brickgrid.fallback_data[idx] = fallback;
    }

    pub(crate) fn rebuild(&mut self, brickgrid_size: u32) {
        self.brickgrid = Brickgrid::new(brickgrid_size);

        let size = brickgrid_size as usize;
        let camera_pos = self.camera_position.clone();

        for x in 0..size {
            for y in 0..size {
                for z in 0..size {
                    self.refresh_brickgrid_cell(x, y, z, &camera_pos);
                }
            }
        }
    }
}
