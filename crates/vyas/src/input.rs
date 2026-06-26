use std::collections::{HashMap, HashSet};

use crate::ecs::Res;

pub type Input<'a> = Res<'a, InputState>;

pub type KeyCode = winit::keyboard::KeyCode;

pub type MouseButton = winit::event::MouseButton;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputButton {
    Key(KeyCode),
    Mouse(MouseButton),
}

impl From<KeyCode> for InputButton {
    fn from(key: KeyCode) -> Self {
        Self::Key(key)
    }
}

impl From<MouseButton> for InputButton {
    fn from(button: MouseButton) -> Self {
        Self::Mouse(button)
    }
}

#[derive(Default)]
pub struct InputState {
    inputs: HashMap<InputButton, bool>,
    just_pressed: HashSet<InputButton>,
}

impl InputState {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn insert(&mut self, input_button: InputButton, pressed: bool) {
        if pressed {
            self.just_pressed.insert(input_button);
        }

        self.inputs.insert(input_button, pressed);
    }

    pub fn pressed(&self, input_button: InputButton) -> bool {
        self.inputs.get(&input_button).is_some_and(|v| *v)
    }

    pub fn just_pressed(&self, input_button: InputButton) -> bool {
        self.just_pressed.contains(&input_button)
    }

    pub fn update(&mut self) {
        self.just_pressed.clear();
    }
}
