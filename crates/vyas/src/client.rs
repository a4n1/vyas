use std::{cell::RefCell, rc::Rc, sync::Arc};

use winit::{dpi::PhysicalPosition, event::MouseScrollDelta, window::Window};

use crate::{
    app::AppConfig,
    ecs::{Res, ResMut, Resource},
    engine::Engine,
    input::InputButton,
};

type ClientCommand = Box<dyn FnOnce(&mut Client)>;

#[derive(Clone, Default)]
pub struct SharedClient {
    client: Rc<RefCell<Option<Client>>>,
    commands: Rc<RefCell<Vec<ClientCommand>>>,
}

impl SharedClient {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn send(&self, command: impl FnOnce(&mut Client) + 'static) {
        self.commands.borrow_mut().push(Box::new(command));
    }

    pub fn set_resource<T: Resource>(&self, resource: T) {
        self.send(move |client| {
            *client.resource_mut::<T>() = resource;
        });
    }

    pub(crate) fn set(&self, client: Client) {
        *self.client.borrow_mut() = Some(client);
    }

    pub(crate) fn with_mut(&self, f: impl FnOnce(&mut Client)) {
        let mut client = self.client.borrow_mut();

        let Some(client) = client.as_mut() else {
            return;
        };

        f(client);
    }

    pub(crate) fn apply_commands(&self) {
        if self.client.borrow().is_none() {
            return;
        }

        let commands = self.commands.borrow_mut().drain(..).collect::<Vec<_>>();

        if commands.is_empty() {
            return;
        }

        self.with_mut(|client| {
            for command in commands {
                command(client);
            }
        });
    }
}

pub struct Client {
    engine: Engine,
}

impl Client {
    pub(crate) async fn new(app_config: AppConfig, window: Arc<Window>) -> Self {
        let engine = Engine::new(app_config, window).await;

        Self { engine }
    }

    pub(crate) fn window(&self) -> &Window {
        &self.engine.graphics.window
    }

    pub(crate) fn request_redraw(&self) {
        self.window().request_redraw();
    }

    pub(crate) fn render(&mut self) {
        self.engine.update();
        self.engine.render();
    }

    pub(crate) fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.engine.resize(new_size);
    }

    pub(crate) fn handle_button_press(&mut self, button: InputButton, pressed: bool) {
        self.engine.handle_button_press(button, pressed);
    }

    pub(crate) fn handle_mouse_move(&mut self, position: PhysicalPosition<f64>) {
        self.engine.handle_mouse_move(position);
    }

    pub(crate) fn handle_mouse_scroll(&mut self, delta: MouseScrollDelta) {
        self.engine.handle_mouse_scroll(delta);
    }

    pub fn resource<T: Resource>(&mut self) -> Res<'_, T> {
        self.engine.resource::<T>()
    }

    pub fn resource_mut<T: Resource>(&mut self) -> ResMut<'_, T> {
        self.engine.resource_mut::<T>()
    }
}
