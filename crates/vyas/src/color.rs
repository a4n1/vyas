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
}

#[derive(Debug, Clone)]
pub struct Srgb {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}
