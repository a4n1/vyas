use serde::{Deserialize, Serialize};

use crate::color::PaletteColor;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Voxel {
    pub palette_color: PaletteColor,
}
