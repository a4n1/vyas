use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::PhysicalKey,
    window::Window,
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use winit::platform::web::EventLoopExtWebSys;

use crate::{
    camera::CameraConfig,
    client::{Client, SharedClient},
    config::RenderConfig,
    ecs::{IntoSystem, Resource, Schedule, System, World},
};

#[cfg(target_arch = "wasm32")]
const CANVAS_ID: &str = "vyas";

pub struct App {
    #[cfg(target_arch = "wasm32")]
    proxy: Option<winit::event_loop::EventLoopProxy<Client>>,
    client: SharedClient,
    config: Option<AppConfig>,
}

#[derive(Default)]
pub(crate) struct AppConfig {
    pub(crate) camera_config: CameraConfig,
    pub(crate) render_config: RenderConfig,
    pub(crate) systems: Vec<(Schedule, Box<dyn System>)>,
    pub(crate) resources: Vec<Box<dyn FnOnce(&mut World)>>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            #[cfg(target_arch = "wasm32")]
            proxy: None,
            client: SharedClient::new(),
            config: Some(AppConfig::default()),
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_client(mut self, client: Option<SharedClient>) -> Self {
        if let Some(client) = client {
            self.client = client;
        }

        self
    }

    pub fn set_camera(mut self, camera_config: CameraConfig) -> Self {
        let config = self.config.as_mut().expect("app config already taken");
        config.camera_config = camera_config;
        self
    }

    pub fn set_render_config(mut self, render_config: RenderConfig) -> Self {
        let config = self.config.as_mut().expect("app config already taken");
        config.render_config = render_config;
        self
    }

    pub fn insert_resource(mut self, resource: impl Resource) -> Self {
        let config = self.config.as_mut().expect("app config already taken");
        config
            .resources
            .push(Box::new(move |world| world.insert_resource(resource)));
        self
    }

    pub fn add_systems<S, Params>(mut self, schedule: Schedule, system: S) -> Self
    where
        S: IntoSystem<Params>,
        S::System: 'static,
    {
        let config = self.config.as_mut().expect("app config already taken");
        config
            .systems
            .push((schedule, Box::new(system.into_system())));
        self
    }

    pub fn run(mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        env_logger::init();

        #[cfg(target_arch = "wasm32")]
        console_log::init_with_level(log::Level::Info).unwrap_throw();

        let event_loop = EventLoop::with_user_event()
            .build()
            .expect("failed to build event loop");

        #[cfg(not(target_arch = "wasm32"))]
        event_loop.run_app(&mut self).expect("failed to run app");

        #[cfg(target_arch = "wasm32")]
        {
            self.proxy = Some(event_loop.create_proxy());
            event_loop.spawn_app(self);
        }
    }
}

impl ApplicationHandler<Client> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        #[allow(unused_mut)]
        let mut window_attributes = Window::default_attributes();

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use winit::platform::web::WindowAttributesExtWebSys;

            let window = wgpu::web_sys::window().unwrap_throw();
            let document = window.document().unwrap_throw();
            let canvas = document.get_element_by_id(CANVAS_ID).unwrap_throw();
            let html_canvas_element = canvas.unchecked_into();
            window_attributes = window_attributes.with_canvas(Some(html_canvas_element));
        }

        let window = Arc::new(
            event_loop
                .create_window(window_attributes)
                .expect("failed to create window"),
        );

        let config = self.config.take().expect("app config already taken");

        #[cfg(not(target_arch = "wasm32"))]
        {
            self.client
                .set(pollster::block_on(Client::new(config, window)));
            self.client.apply_commands();
        }

        #[cfg(target_arch = "wasm32")]
        {
            if let Some(proxy) = self.proxy.take() {
                wasm_bindgen_futures::spawn_local(async move {
                    assert!(proxy.send_event(Client::new(config, window).await).is_ok())
                });
            }
        }
    }

    #[allow(unused_mut)]
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, mut client: Client) {
        #[cfg(target_arch = "wasm32")]
        {
            client.window().request_redraw();
            client.resize(client.window().inner_size());
        }

        self.client.set(client);
        self.client.apply_commands();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        self.client.apply_commands();

        self.client.with_mut(|client| match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                client.resize(size);
                client.request_redraw();
            }
            WindowEvent::Focused(true) | WindowEvent::Occluded(false) => client.request_redraw(),
            WindowEvent::RedrawRequested => {
                client.render();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state,
                        ..
                    },
                ..
            } => {
                client.handle_button_press(code.into(), state.is_pressed());
            }
            WindowEvent::MouseInput { button, state, .. } => {
                client.handle_button_press(button.into(), state.is_pressed());
            }
            WindowEvent::CursorMoved { position, .. } => {
                client.handle_mouse_move(position);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                client.handle_mouse_scroll(delta);
            }
            _ => {}
        });
    }
}
