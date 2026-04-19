use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use winit::platform::web::EventLoopExtWebSys;

use crate::{engine::Engine, graphics::Graphics};

#[derive(Default)]
pub struct App {
    #[cfg(target_arch = "wasm32")]
    proxy: Option<winit::event_loop::EventLoopProxy<Client>>,
    client: Option<Client>,
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn run(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        env_logger::init();

        #[cfg(target_arch = "wasm32")]
        console_log::init_with_level(log::Level::Info).unwrap_throw();

        let event_loop = EventLoop::with_user_event()
            .build()
            .expect("failed to build event loop");

        #[cfg(not(target_arch = "wasm32"))]
        event_loop.run_app(self).expect("failed to run app");

        #[cfg(target_arch = "wasm32")]
        {
            let mut app = App::new();
            app.proxy = Some(event_loop.create_proxy());
            event_loop.spawn_app(app);
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

            const CANVAS_ID: &str = "canvas";

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

        #[cfg(not(target_arch = "wasm32"))]
        {
            self.client = Some(pollster::block_on(Client::new(window)));
        }

        #[cfg(target_arch = "wasm32")]
        {
            if let Some(proxy) = self.proxy.take() {
                wasm_bindgen_futures::spawn_local(async move {
                    assert!(proxy.send_event(Client::new(window).await).is_ok())
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
        self.client = Some(client);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let client = match &mut self.client {
            Some(client) => client,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => client.resize(size),
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
            } => match (code, state.is_pressed()) {
                (KeyCode::Escape, true) => event_loop.exit(),
                (code, is_pressed) => client.handle_key(code, is_pressed),
            },
            _ => {}
        }
    }
}

pub struct Client {
    graphics: Graphics,
    engine: Engine,
}

impl Client {
    async fn new(window: Arc<Window>) -> Self {
        let graphics = Graphics::new(window).await;
        let engine = Engine::new(&graphics);

        Self { graphics, engine }
    }

    #[cfg(target_arch = "wasm32")]
    fn window(&self) -> &Window {
        self.graphics.window()
    }

    fn render(&mut self) {
        self.engine.update(&mut self.graphics);
        self.engine.render(&self.graphics);
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.engine.resize(new_size, &mut self.graphics);
    }

    fn handle_key(&mut self, code: KeyCode, is_pressed: bool) {
        self.engine.handle_key(code, is_pressed, &mut self.graphics);
    }
}
