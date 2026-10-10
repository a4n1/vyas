use crate::{
    color::{Color, Srgb},
    ecs::ResMut,
};

pub(crate) const BRICKMAP_SIZE: u32 = 8;
pub(crate) const FALLBACK_BRICKMAP_SIZE: u32 = 2;

pub type RenderConfig<'a> = ResMut<'a, RenderConfigState>;

#[derive(Debug, Clone, Copy)]
pub struct RenderConfigState {
    pub brickgrid_size: u32,
    brickmap_size: u32,
    pub fill_color: Color,
    pub resolution: Resolution,
    pub vsync: bool,
    pub crt: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Resolution {
    Scale(f64),
    Dimensions { width: u32, height: u32 },
}

impl Default for Resolution {
    fn default() -> Self {
        Self::Scale(1.0)
    }
}

impl Default for RenderConfigState {
    fn default() -> Self {
        Self {
            brickgrid_size: 128,
            brickmap_size: BRICKMAP_SIZE,
            fill_color: Color::Srgb(Srgb {
                r: 25,
                g: 25,
                b: 25,
            }),
            resolution: Resolution::default(),
            vsync: true,
            crt: false,
        }
    }
}

impl RenderConfigState {
    pub(crate) fn to_gpu(self, time: f32) -> RenderConfigGpu {
        RenderConfigGpu {
            brickmap_size: self.brickmap_size,
            brickgrid_size: self.brickgrid_size,
            crt: u32::from(self.crt),
            fallback_brickmap_size: FALLBACK_BRICKMAP_SIZE,
            fill_color: self.fill_color.into(),
            time,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RenderConfigGpu {
    brickmap_size: u32,
    brickgrid_size: u32,
    crt: u32,
    fallback_brickmap_size: u32,
    fill_color: [f32; 3],
    time: f32,
}

#[derive(Debug, Default)]
pub struct RenderConfigBuilder {
    brickgrid_size: Option<u32>,
    brickmap_size: Option<u32>,
    fill_color: Option<Color>,
    resolution: Option<Resolution>,
    vsync: Option<bool>,
    crt: Option<bool>,
}

impl RenderConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn brickgrid_size(mut self, size: u32) -> Self {
        self.brickgrid_size = Some(size);
        self
    }

    pub fn brickmap_size(mut self, size: u32) -> Self {
        self.brickmap_size = Some(size);
        self
    }

    pub fn fill_color(mut self, color: Color) -> Self {
        self.fill_color = Some(color);
        self
    }

    pub fn resolution(mut self, resolution: Resolution) -> Self {
        self.resolution = Some(resolution);
        self
    }

    pub fn vsync(mut self, value: bool) -> Self {
        self.vsync = Some(value);
        self
    }

    pub fn crt(mut self, value: bool) -> Self {
        self.crt = Some(value);
        self
    }

    pub fn build(self) -> RenderConfigState {
        let default = RenderConfigState::default();

        RenderConfigState {
            brickgrid_size: self.brickgrid_size.unwrap_or(default.brickgrid_size),
            brickmap_size: self.brickmap_size.unwrap_or(default.brickmap_size),
            fill_color: self.fill_color.unwrap_or(default.fill_color),
            resolution: self.resolution.unwrap_or(default.resolution),
            vsync: self.vsync.unwrap_or(default.vsync),
            crt: self.crt.unwrap_or(default.crt),
        }
    }
}
