use std::{
    cell::RefMut,
    collections::{HashMap, HashSet},
};

use crate::{
    brickmap::Brickmap,
    camera::CameraState,
    config::RenderConfigState,
    ecs::{Entity, World},
    position::GridPosition,
};

pub(crate) struct Brickgrid {
    pub(crate) map: HashMap<GridPosition, Entity>,
}

impl Brickgrid {
    pub(crate) fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub(crate) fn get(&self, position: &GridPosition) -> Option<&Entity> {
        self.map.get(position)
    }

    pub(crate) fn insert(&mut self, position: GridPosition, entity: Entity) {
        self.map.insert(position, entity);
    }

    pub(crate) fn visible_brickmaps<'a>(
        &'a self,
        world: &'a World,
    ) -> impl Iterator<Item = VisibleBrickmap<'a>> {
        let camera_brickmap_position = world
            .resource::<CameraState>()
            .position
            .to_brickmap_position();
        let render_config = *world.resource::<RenderConfigState>();

        let brickgrid_size = render_config.brickgrid_size as i32;
        let range = (brickgrid_size - 1) / 2;

        self.map.iter().filter_map(move |(position, entity)| {
            let x = position.x - camera_brickmap_position.x;
            let y = camera_brickmap_position.y - position.y;
            let z = camera_brickmap_position.z - position.z;

            if !(-range..=range).contains(&x)
                || !(-range..=range).contains(&y)
                || !(-range..=range).contains(&z)
            {
                return None;
            }

            let index = GridPosition {
                x: x + range,
                y: y + range,
                z: z + range,
            }
            .to_index(brickgrid_size);

            let brickmap = world
                .get_mut::<Brickmap>(*entity)
                .expect("entity should have a brickmap component");

            Some(VisibleBrickmap {
                brickgrid_index: index,
                brickmap,
            })
        })
    }
}

pub(crate) struct VisibleBrickmap<'a> {
    pub(crate) brickmap: RefMut<'a, Brickmap>,
    pub(crate) brickgrid_index: usize,
}

/// Packed 32-bit cell representation:
///
/// Loaded:
///   0              = empty
///   bits 0..=23    = brickmap index
///
/// Unloaded:
///   bit 24         = unloaded flag
///   bit 25         = requested flag
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct BrickgridCell(u32);

const UNLOADED_BIT: u32 = 1 << 24;
const REQUESTED_BIT: u32 = 1 << 25;

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub(crate) enum BrickgridCellData {
    #[default]
    Empty,
    Unloaded,
    Requested,
    Loaded {
        brickmap_index: u32,
    },
}

impl From<BrickgridCellData> for BrickgridCell {
    fn from(value: BrickgridCellData) -> Self {
        match value {
            BrickgridCellData::Empty => BrickgridCell(0),
            BrickgridCellData::Unloaded => BrickgridCell(UNLOADED_BIT),
            BrickgridCellData::Requested => BrickgridCell(UNLOADED_BIT | REQUESTED_BIT),
            BrickgridCellData::Loaded { brickmap_index } => BrickgridCell(brickmap_index),
        }
    }
}

impl From<BrickgridCell> for BrickgridCellData {
    fn from(value: BrickgridCell) -> Self {
        let value = value.0;

        if value == 0 {
            return BrickgridCellData::Empty;
        }

        let unloaded = value & UNLOADED_BIT != 0;
        let requested = value & REQUESTED_BIT != 0;

        if unloaded {
            if requested {
                BrickgridCellData::Requested
            } else {
                BrickgridCellData::Unloaded
            }
        } else {
            BrickgridCellData::Loaded {
                brickmap_index: value,
            }
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct FeedbackGpu {
    count: u32,
    data: [u32; 4096],
}

#[derive(Debug, Default)]
pub(crate) struct Feedback {
    data: HashSet<u32>,
}

impl Feedback {
    pub(crate) fn is_requested(&self, position: u32) -> bool {
        self.data.contains(&position)
    }
}

impl From<&FeedbackGpu> for Feedback {
    fn from(value: &FeedbackGpu) -> Self {
        let count = (value.count as usize).min(value.data.len());

        Self {
            data: HashSet::from_iter(value.data[..count].iter().copied()),
        }
    }
}
