#[derive(Debug, Clone)]
pub enum Color {
    Srgb(Srgb),
}

impl Color {
    pub fn r(&self) -> u8 {
        match self {
            Color::Srgb(srgb) => srgb.r,
        }
    }

    pub fn g(&self) -> u8 {
        match self {
            Color::Srgb(srgb) => srgb.g,
        }
    }

    pub fn b(&self) -> u8 {
        match self {
            Color::Srgb(srgb) => srgb.b,
        }
    }

    pub(crate) fn linear_rgb(&self) -> u32 {
        match self {
            Color::Srgb(srgb) => {
                ((Self::srgb_component_to_linear(srgb.r) as u32) << 16)
                    | ((Self::srgb_component_to_linear(srgb.g) as u32) << 8)
                    | (Self::srgb_component_to_linear(srgb.b) as u32)
            }
        }
    }

    fn srgb_component_to_linear(value: u8) -> u8 {
        let value = value as f32 / 255.0;

        let linear = if value <= 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        };

        (linear * 255.0).round() as u8
    }
}

#[derive(Debug, Clone)]
pub struct Srgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}
