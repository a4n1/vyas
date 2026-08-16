use std::{borrow::Cow, collections::HashMap, fmt::Display};

use serde::{Deserialize, Serialize};

use crate::ecs::ResMut;

pub type ColorPalette<'a> = ResMut<'a, ColorPaletteState>;

#[derive(Clone, Serialize, Deserialize)]
pub struct ColorPaletteState {
    palette: HashMap<String, ColorPaletteItem>,
    available_entries: Vec<u8>,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct ColorPaletteItem {
    pub(crate) color: Color,
    pub(crate) index: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorPaletteError {
    PaletteFull,
}

impl ColorPaletteState {
    pub(crate) fn new() -> Self {
        Self {
            palette: HashMap::new(),
            available_entries: (0..=u8::MAX).collect(),
        }
    }

    pub fn insert(&mut self, palette_color: PaletteColor) -> Result<(), ColorPaletteError> {
        if self.palette.contains_key(&palette_color.name.to_string()) {
            return Ok(());
        }

        let index = self
            .available_entries
            .pop()
            .ok_or(ColorPaletteError::PaletteFull)?;

        self.palette.insert(
            palette_color.name.to_string(),
            ColorPaletteItem {
                color: palette_color.color,
                index,
            },
        );

        Ok(())
    }

    pub fn remove(&mut self, palette_color: PaletteColor) {
        if let Some(item) = self.palette.remove(&palette_color.name.to_string()) {
            self.available_entries.push(item.index);
        }
    }

    pub(crate) fn get(&self, palette_color: &PaletteColor) -> Option<&ColorPaletteItem> {
        self.palette.get(&palette_color.name.to_string())
    }

    pub fn clear(&mut self) {
        *self = Self::new()
    }

    pub fn to_gpu(&self) -> Vec<u32> {
        let mut colors = vec![0; 256];

        for item in self.palette.values() {
            colors[item.index as usize] = u32::from(item.color);
        }

        colors
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PaletteColor {
    name: Cow<'static, str>,
    pub(crate) color: Color,
}

impl PaletteColor {
    pub const fn new_static(name: &'static str, color: Color) -> Self {
        Self {
            name: Cow::Borrowed(name),
            color,
        }
    }

    pub fn new(name: impl Into<Cow<'static, str>>, color: Color) -> Self {
        Self {
            name: name.into(),
            color,
        }
    }

    pub fn name(&self) -> String {
        self.name.to_string()
    }
}

impl From<Color> for PaletteColor {
    fn from(value: Color) -> Self {
        PaletteColor::new(value.to_string(), value)
    }
}

#[macro_export]
macro_rules! palette_color {
    ($name:ident, $value:expr) => {
        pub(crate) const $name: PaletteColor =
            PaletteColor::new_static(stringify!($name), Color::Srgb($value));
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},{},{}", self.r(), self.g(), self.b())
    }
}

impl From<u32> for Color {
    fn from(value: u32) -> Self {
        Color::Srgb(Srgb {
            r: (value & 0xFF) as u8,
            g: ((value >> 8) & 0xFF) as u8,
            b: ((value >> 16) & 0xFF) as u8,
        })
    }
}

impl From<Color> for u32 {
    fn from(value: Color) -> Self {
        (value.r() as u32)
            | ((value.g() as u32) << 8)
            | ((value.b() as u32) << 16)
            | ((0xFFu32) << 24)
    }
}

impl From<Color> for [f32; 3] {
    fn from(value: Color) -> Self {
        let r = srgb_to_linear(value.r());
        let g = srgb_to_linear(value.g());
        let b = srgb_to_linear(value.b());

        [r, g, b]
    }
}

fn srgb_to_linear(value: u8) -> f32 {
    let value = value as f32 / u8::MAX as f32;

    if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Srgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl From<Srgb> for [u8; 4] {
    fn from(value: Srgb) -> Self {
        [value.r, value.g, value.b, u8::MAX]
    }
}

impl From<Srgb> for u32 {
    fn from(value: Srgb) -> Self {
        Color::Srgb(value).into()
    }
}
