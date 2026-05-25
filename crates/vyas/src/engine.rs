use std::{cell::RefCell, sync::Arc};

use winit::{dpi::PhysicalSize, keyboard::KeyCode, window::Window};

use crate::{
    app::AppConfig,
    camera::CameraState,
    chunk::ChunkMap,
    ecs::{CommandQueue, Schedule, System, World},
    fps::FpsCounter,
    graphics::Graphics,
    input::InputState,
    pipeline::Pipeline,
};

pub(crate) struct Engine {
    pub(crate) graphics: Graphics,
    world: World,
    pipeline: Pipeline,
    fps_counter: FpsCounter,
    systems: Vec<(Schedule, Box<dyn System>)>,
    started: bool,
}

impl Engine {
    pub(crate) async fn new(app_config: AppConfig, window: Arc<Window>) -> Self {
        let AppConfig {
            camera_config,
            render_config,
            systems,
        } = app_config;

        let mut world = World::new();
        world.insert_resource(InputState::new());
        world.insert_resource(CameraState::new(camera_config));
        world.insert_resource(render_config);
        world.insert_resource(ChunkMap::new());

        let graphics = Graphics::new(window, &render_config).await;
        let pipeline = Pipeline::new(&graphics, &world);
        let fps_counter = FpsCounter::new();

        Self {
            graphics,
            world,
            pipeline,
            fps_counter,
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

        self.pipeline.update(&self.world);
        self.graphics.update(&self.pipeline, &self.world);
    }

    pub(crate) fn render(&mut self) {
        self.graphics.render(&self.pipeline);
        self.fps_counter.tick();
    }

    pub(crate) fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            return;
        }

        self.graphics.resize(size);
        self.pipeline.resize(&self.graphics);

        let mut camera = self.world.resource_mut::<CameraState>();
        camera.resize(&self.graphics.surface_config);
    }

    pub(crate) fn handle_key_press(&mut self, code: KeyCode, pressed: bool) {
        let mut button_state = self.world.resource_mut::<InputState>();
        button_state.upsert(code, pressed);
    }
}
