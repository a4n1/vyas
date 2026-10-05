use std::collections::HashSet;

#[derive(Debug, Clone)]
pub(crate) struct Brickgrid {
    pub(crate) data: Vec<BrickgridCell>,
}

impl Brickgrid {
    pub(crate) fn new(size: u32) -> Self {
        Brickgrid {
            data: vec![BrickgridCellData::Empty.into(); size.pow(3) as usize],
        }
    }

    pub(crate) fn size(&self) -> usize {
        (self.data.len() as f32).cbrt() as usize
    }
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
    pub(crate) data: HashSet<u32>,
}

impl From<&FeedbackGpu> for Feedback {
    fn from(value: &FeedbackGpu) -> Self {
        let count = (value.count as usize).min(value.data.len());

        Self {
            data: HashSet::from_iter(value.data[..count].iter().copied()),
        }
    }
}
