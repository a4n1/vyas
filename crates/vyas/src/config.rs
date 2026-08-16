use crate::{
    color::{Color, Srgb},
    ecs::ResMut,
};

pub(crate) const BRICKMAP_SIZE: u32 = 8;

pub type RenderConfig<'a> = ResMut<'a, RenderConfigState>;

#[derive(Clone, Copy, Debug)]
pub struct RenderConfigState {
    pub brickgrid_size: u32,
    brickmap_size: u32,
    pub fill_color: Color,
}

impl Default for RenderConfigState {
    fn default() -> Self {
        Self {
            brickgrid_size: 257,
            brickmap_size: BRICKMAP_SIZE,
            fill_color: Color::Srgb(Srgb {
                r: 25,
                g: 25,
                b: 25,
            }),
        }
    }
}

impl RenderConfigState {
    pub(crate) fn to_gpu(self) -> RenderConfigGpu {
        RenderConfigGpu {
            brickmap_size: self.brickmap_size,
            brickgrid_size: self.brickgrid_size,
            _padding0: [0; 2],
            fill_color: self.fill_color.into(),
            _padding1: 0,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RenderConfigGpu {
    pub brickmap_size: u32,
    pub brickgrid_size: u32,
    _padding0: [u32; 2],
    pub fill_color: [f32; 3],
    _padding1: u32,
}
