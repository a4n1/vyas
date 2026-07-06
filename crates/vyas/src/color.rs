#[derive(Debug, Clone)]
pub enum Color {
    Srgb(Srgb),
}

impl Color {
    pub fn r(&self) -> f32 {
        match self {
            Color::Srgb(srgb) => srgb.r,
        }
    }

    pub fn g(&self) -> f32 {
        match self {
            Color::Srgb(srgb) => srgb.g,
        }
    }

    pub fn b(&self) -> f32 {
        match self {
            Color::Srgb(srgb) => srgb.b,
        }
    }

    pub(crate) fn linear_rgb(&self) -> [f32; 3] {
        match self {
            Color::Srgb(srgb) => [
                Self::srgb_component_to_linear(srgb.r),
                Self::srgb_component_to_linear(srgb.g),
                Self::srgb_component_to_linear(srgb.b),
            ],
        }
    }

    fn srgb_component_to_linear(value: f32) -> f32 {
        if value <= 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    }
}

#[derive(Debug, Clone)]
pub struct Srgb {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}
