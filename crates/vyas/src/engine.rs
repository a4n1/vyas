use std::{cell::RefCell, sync::Arc};

#[cfg(feature = "egui")]
use winit::event::WindowEvent;
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::MouseScrollDelta,
    window::Window,
};

use crate::{
    app::AppConfig,
    camera::CameraState,
    color::ColorPaletteState,
    ecs::{CommandQueue, Res, ResMut, Resource, Schedule, System, World},
    fps::Fps,
    graphics::Graphics,
    input::{InputButton, InputState},
    pipeline::Pipeline,
    scene::SceneState,
};

pub(crate) struct Engine {
    pub(crate) graphics: Graphics,
    world: World,
    pipeline: Pipeline,
    systems: Vec<(Schedule, Box<dyn System>)>,
    started: bool,
}

impl Engine {
    pub(crate) async fn new(app_config: AppConfig, window: Arc<Window>) -> Self {
        let AppConfig {
            camera_config,
            render_config,
            systems,
            resources,
            ..
        } = app_config;

        let mut world = World::new();

        let camera_pos = camera_config.position.clone();

        world.insert_resource(InputState::new());
        world.insert_resource(CameraState::new(camera_config));
        world.insert_resource(render_config);
        world.insert_resource(SceneState::new(
            render_config.brickgrid_size,
            camera_pos.to_brickmap_position(),
        ));
        world.insert_resource(ColorPaletteState::new());
        world.insert_resource(Fps::new());

        for insert_resource in resources {
            insert_resource(&mut world);
        }

        #[cfg(feature = "egui")]
        let graphics = Graphics::new(window, &render_config, app_config.egui).await;
        #[cfg(not(feature = "egui"))]
        let graphics = Graphics::new(window, &render_config).await;
        let pipeline = Pipeline::new(&graphics, &world);

        Self {
            graphics,
            world,
            pipeline,
            systems,
            started: false,
        }
    }

    pub(crate) fn update(&mut self) {
        let command_queue: CommandQueue = RefCell::new(Vec::new());

        if !self.started {
            for (schedule, system) in &mut self.systems {
                if matches!(schedule, Schedule::Startup) {
                    system.run(&self.world, &command_queue);
                }
            }

            self.started = true;
        }

        for (schedule, system) in &mut self.systems {
            if matches!(schedule, Schedule::Update) {
                system.run(&self.world, &command_queue);
            }
        }

        for command in command_queue.into_inner() {
            command(&mut self.world);
        }

        self.pipeline.update(&self.graphics, &self.world);
        self.graphics.update(&self.pipeline, &self.world);

        let mut input_state = self.world.resource_mut::<InputState>();
        input_state.update();
    }

    pub(crate) fn render(&mut self) {
        self.graphics.render(&self.pipeline, &self.world);
        self.world.resource_mut::<Fps>().tick();
    }

    pub(crate) fn resize(&mut self, size: PhysicalSize<u32>) {
        self.graphics.resize(size);

        if size.width == 0 || size.height == 0 {
            return;
        }

        self.pipeline.resize(&self.graphics);

        let mut camera = self.world.resource_mut::<CameraState>();
        camera.resize(&self.graphics.surface_config);
    }

    pub(crate) fn handle_button_press(&mut self, button: InputButton, pressed: bool) {
        let mut input_state = self.world.resource_mut::<InputState>();
        input_state.insert_pressed(button, pressed);
    }

    #[cfg(feature = "egui")]
    pub(crate) fn handle_window_event(&mut self, event: &WindowEvent) -> bool {
        let graphics = &mut self.graphics;
        let response = graphics.egui.state.on_window_event(&graphics.window, event);

        if response.repaint {
            graphics.window.request_redraw();
        }

        response.consumed
    }

    #[cfg(feature = "egui")]
    pub(crate) fn egui_wants_pointer_input(&self) -> bool {
        self.graphics
            .egui
            .state
            .egui_ctx()
            .egui_wants_pointer_input()
    }

    pub(crate) fn handle_mouse_move(&mut self, position: PhysicalPosition<f64>) {
        let mut input_state = self.world.resource_mut::<InputState>();
        input_state.set_mouse_position(position, &self.world, self.graphics.viewport_size);
    }

    pub(crate) fn handle_mouse_motion(&mut self, delta: (f64, f64)) {
        let mut input_state = self.world.resource_mut::<InputState>();
        input_state.increment_mouse_delta(delta);
    }

    pub(crate) fn handle_mouse_scroll(&mut self, delta: MouseScrollDelta) {
        let mut input_state = self.world.resource_mut::<InputState>();
        input_state.set_scroll_delta(delta);
    }

    pub(crate) fn resource<T: Resource>(&mut self) -> Res<'_, T> {
        self.world.resource::<T>()
    }

    pub(crate) fn resource_mut<T: Resource>(&mut self) -> ResMut<'_, T> {
        self.world.resource_mut::<T>()
    }
}
