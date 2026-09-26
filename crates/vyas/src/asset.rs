use vox_rs::VoxError;

use crate::color::{ColorPaletteError, ColorPaletteState};
use crate::prelude::GridPosition;
use crate::voxel::Voxel;

pub struct VoxAsset {
    pub palette: ColorPaletteState,
    pub grid: Grid,
}

pub type Grid = Vec<Block>;

pub struct Block {
    pub position: GridPosition,
    pub voxel: Voxel,
}

#[derive(Debug, Clone)]
pub enum VoxAssetError {
    Vox(VoxError),
    ColorPalette(ColorPaletteError),
}

impl From<VoxError> for VoxAssetError {
    fn from(value: VoxError) -> Self {
        Self::Vox(value)
    }
}

impl From<ColorPaletteError> for VoxAssetError {
    fn from(value: ColorPaletteError) -> Self {
        Self::ColorPalette(value)
    }
}

pub mod vox {
    use vox_rs::Scene;

    use crate::color::{Color, ColorPaletteState, PaletteColor, Srgb};
    use crate::prelude::GridPosition;
    use crate::voxel::Voxel;

    use super::*;

    pub fn load(bytes: &[u8]) -> Result<VoxAsset, VoxAssetError> {
        let scene = Scene::read_bytes(bytes)?;

        let mut palette = ColorPaletteState::new();

        for color in scene.palette.colors {
            palette.insert(PaletteColor::from(Color::Srgb(Srgb {
                r: color.r,
                g: color.g,
                b: color.b,
            })))?;
        }

        let mut grid: Vec<Block> = Vec::new();

        for instance in scene.instances.iter().filter(|instance| {
            !instance.hidden
                && instance
                    .layer_index
                    .and_then(|index| scene.layers.get(index))
                    .is_none_or(|layer| !layer.hidden)
        }) {
            let model = &scene.models[instance.model_index];
            let sx = model.size_x as usize;
            let sy = model.size_y as usize;

            for (i, &color_index) in model.voxels.iter().enumerate() {
                if color_index == 0 {
                    continue;
                }

                let x = (i % sx) as f32;
                let y = ((i / sx) % sy) as f32;
                let z = (i / (sx * sy)) as f32;
                let transform = &instance.transform;
                let scene_x =
                    x * transform.m00 + y * transform.m10 + z * transform.m20 + transform.m30;
                let scene_y =
                    x * transform.m01 + y * transform.m11 + z * transform.m21 + transform.m31;
                let scene_z =
                    x * transform.m02 + y * transform.m12 + z * transform.m22 + transform.m32;

                let color = scene.palette.colors[color_index as usize];
                let palette_color = PaletteColor::from(Color::Srgb(Srgb {
                    r: color.r,
                    g: color.g,
                    b: color.b,
                }));

                grid.push(Block {
                    // .vox is z-up
                    position: GridPosition {
                        x: scene_x.round() as i32,
                        y: scene_z.round() as i32,
                        z: scene_y.round() as i32,
                    },
                    voxel: Voxel { palette_color },
                });
            }
        }

        Ok(VoxAsset { palette, grid })
    }

    pub fn load_with_palette(
        bytes: &[u8],
        palette: &mut ColorPaletteState,
    ) -> Result<Grid, VoxAssetError> {
        let asset = load(bytes)?;
        *palette = asset.palette;
        Ok(asset.grid)
    }
}
