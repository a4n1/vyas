use std::collections::HashMap;

use crate::ecs::Res;

pub type Input<'a> = Res<'a, InputState>;

pub type KeyCode = winit::keyboard::KeyCode;

#[derive(Default)]
pub struct InputState {
    inner: HashMap<KeyCode, bool>,
}

impl InputState {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn upsert(&mut self, key_code: KeyCode, pressed: bool) {
        self.inner.insert(key_code, pressed);
    }

    pub fn pressed(&self, key_code: KeyCode) -> bool {
        self.inner.get(&key_code).is_some_and(|v| *v)
    }
}
