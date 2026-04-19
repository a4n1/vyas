use winit::{dpi::PhysicalSize, keyboard::KeyCode};

use crate::graphics::Graphics;

pub struct Engine;

// TODO: do we need to store graphics/renderer

impl Engine {
    pub fn new(_graphics: &Graphics) -> Self {
        Self
    }

    pub fn update(&mut self, graphics: &mut Graphics) {
        graphics.update();
    }

    pub fn render(&self, graphics: &Graphics) {
        graphics.render();
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>, graphics: &mut Graphics) {
        graphics.resize(size);
    }

    pub fn handle_key(&mut self, code: KeyCode, is_pressed: bool, graphics: &mut Graphics) {
        graphics.handle_key(code, is_pressed);
    }
}
