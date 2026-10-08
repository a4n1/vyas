use std::{
    iter,
    sync::{Arc, mpsc},
};

use wgpu::PresentMode;
use winit::{dpi::PhysicalSize, window::Window};

use crate::{
    brickgrid::{Feedback, FeedbackGpu},
    buffer::BrickmapBuffer,
    camera::CameraState,
    color::ColorPaletteState,
    config::RenderConfigState,
    ecs::World,
    pipeline::Pipeline,
    scene::SceneState,
};

pub(crate) struct Graphics {
    pub(crate) window: Arc<Window>,
    pub(crate) device: wgpu::Device,
    pub(crate) surface_config: wgpu::SurfaceConfiguration,
    pub(crate) viewport_size: PhysicalSize<u32>,
    pub(crate) max_texture_dimension_2d: u32,
    pub(crate) supports_srgb_surface_view: bool,
    surface: wgpu::Surface<'static>,
    queue: wgpu::Queue,
    state: State,
}

struct State {
    is_surface_configured: bool,
    brickmap_buffer: BrickmapBuffer,
    feedback_rx: Option<mpsc::Receiver<Result<(), wgpu::BufferAsyncError>>>,
}

impl Graphics {
    pub(crate) async fn new(window: Arc<Window>, render_config: &RenderConfigState) -> Self {
        let size = window.inner_size();

        let instance_descriptor = wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::BROWSER_WEBGPU | wgpu::Backends::GL,
            flags: Default::default(),
            memory_budget_thresholds: Default::default(),
            backend_options: Default::default(),
            display: None,
        };

        #[cfg(not(target_arch = "wasm32"))]
        let instance = wgpu::Instance::new(instance_descriptor);

        #[cfg(target_arch = "wasm32")]
        let instance = wgpu::util::new_instance_with_webgpu_detection(instance_descriptor).await;

        let surface = instance
            .create_surface(window.clone())
            .expect("failed to create surface");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("failed to request adapter");

        let adapter_info = adapter.get_info();
        let supports_srgb_surface_view = adapter_info.backend != wgpu::Backend::Gl;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("device"),
                required_features: wgpu::Features::empty(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                required_limits: wgpu::Limits::default().using_resolution(adapter.limits()),
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await
            .expect("failed to request device");
        let max_texture_dimension_2d = device.limits().max_texture_dimension_2d;

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let render_size = Self::render_size(size, max_texture_dimension_2d);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: if supports_srgb_surface_view {
                surface_format.remove_srgb_suffix()
            } else {
                surface_format
            },
            width: render_size.width,
            height: render_size.height,
            present_mode: Self::get_present_mode(render_config.vsync),
            alpha_mode: surface_caps.alpha_modes[0],
            desired_maximum_frame_latency: 2,
            view_formats: if supports_srgb_surface_view {
                vec![surface_format.add_srgb_suffix()]
            } else {
                vec![]
            },
        };

        Self {
            window,
            device,
            surface_config,
            viewport_size: size,
            max_texture_dimension_2d,
            supports_srgb_surface_view,
            surface,
            queue,
            state: State {
                is_surface_configured: false,
                brickmap_buffer: BrickmapBuffer::new(),
                feedback_rx: None,
            },
        }
    }

    pub(crate) fn resize(&mut self, size: PhysicalSize<u32>) {
        self.viewport_size = size;
        let PhysicalSize { width, height } = Self::render_size(size, self.max_texture_dimension_2d);

        if self.surface_config.width == width
            && self.surface_config.height == height
            && self.state.is_surface_configured
        {
            return;
        }

        self.surface_config.width = width;
        self.surface_config.height = height;
        self.state.is_surface_configured = false;
    }

    pub(crate) fn render_format(&self) -> wgpu::TextureFormat {
        if self.supports_srgb_surface_view {
            self.surface_config.format.add_srgb_suffix()
        } else {
            self.surface_config.format
        }
    }

    pub(crate) fn update(&mut self, pipeline: &Pipeline, world: &World) {
        let render_config = *world.resource::<RenderConfigState>();

        let present_mode = Self::get_present_mode(render_config.vsync);

        if present_mode != self.surface_config.present_mode {
            self.surface_config.present_mode = present_mode;
            self.state.is_surface_configured = false;
        }

        let camera = world.resource::<CameraState>();
        let mut scene = world.resource_mut::<SceneState>();
        let color_palette = world.resource::<ColorPaletteState>();

        let feedback = self.read_feedback(&pipeline.feedback_result_buffer);

        for idx in feedback.data {
            let Some(brickmap) = scene.get_brickmap_by_index(idx) else {
                continue;
            };

            let cell = self
                .state
                .brickmap_buffer
                .insert(brickmap, pipeline, &self.queue);

            if let Some(cell) = cell {
                scene.update_brickgrid(idx, cell);
            }
        }

        scene.update(
            &camera.position.to_brickmap_position(),
            render_config.brickgrid_size,
        );

        self.queue.write_buffer(
            &pipeline.brickgrid_buffer,
            0,
            bytemuck::cast_slice(scene.to_brickgrid_gpu()),
        );

        self.queue.write_buffer(
            &pipeline.camera_buffer,
            0,
            bytemuck::cast_slice(&[pipeline.camera]),
        );

        self.queue.write_buffer(
            &pipeline.render_config_buffer,
            0,
            bytemuck::cast_slice(&[pipeline.render_config.to_gpu()]),
        );

        self.queue.write_buffer(
            &pipeline.color_palette_buffer,
            0,
            bytemuck::cast_slice(&color_palette.to_gpu()),
        );
    }

    fn read_feedback(&mut self, buffer: &wgpu::Buffer) -> Feedback {
        let _ = self.device.poll(wgpu::PollType::Poll);

        let result = self
            .state
            .feedback_rx
            .as_ref()
            .map(|receiver| receiver.try_recv());

        match result {
            Some(Ok(Ok(()))) => {
                let mapped = buffer.slice(..).get_mapped_range();
                let feedback_gpu = bytemuck::from_bytes::<FeedbackGpu>(&mapped);
                let feedback = Feedback::from(feedback_gpu);

                drop(mapped);
                buffer.unmap();
                self.state.feedback_rx = None;

                feedback
            }
            Some(Ok(Err(error))) => {
                log::warn!("failed to map feedback buffer: {error}");
                self.state.feedback_rx = None;
                Feedback::default()
            }
            Some(Err(mpsc::TryRecvError::Disconnected)) => {
                log::warn!("feedback buffer mapping channel disconnected");
                self.state.feedback_rx = None;
                Feedback::default()
            }
            Some(Err(mpsc::TryRecvError::Empty)) | None => Feedback::default(),
        }
    }

    pub(crate) fn render(&mut self, pipeline: &Pipeline) {
        if self.surface_config.width == 0 || self.surface_config.height == 0 {
            return;
        }

        if !self.state.is_surface_configured {
            self.surface.configure(&self.device, &self.surface_config);
            self.state.is_surface_configured = true;
        }

        let output = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(surface_texture) => surface_texture,
            wgpu::CurrentSurfaceTexture::Suboptimal(surface_texture) => {
                self.state.is_surface_configured = false;
                surface_texture
            }
            wgpu::CurrentSurfaceTexture::Timeout
            | wgpu::CurrentSurfaceTexture::Occluded
            | wgpu::CurrentSurfaceTexture::Validation => {
                return;
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.state.is_surface_configured = false;
                return;
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                panic!("Lost device");
            }
        };

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor {
            format: self
                .supports_srgb_surface_view
                .then(|| self.render_format()),
            ..Default::default()
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render-encoder"),
            });

        encoder.clear_buffer(&pipeline.feedback_buffer, 0, None);

        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("raytrace-pass"),
            timestamp_writes: None,
        });

        compute_pass.set_pipeline(&pipeline.compute_pipeline);
        compute_pass.set_bind_group(0, &pipeline.scene_bind_group, &[]);
        let output_size = pipeline.output_size();
        compute_pass.dispatch_workgroups(
            output_size.width.div_ceil(8),
            output_size.height.div_ceil(8),
            1,
        );

        drop(compute_pass);

        let read_feedback = self.state.feedback_rx.is_none();

        if read_feedback {
            encoder.copy_buffer_to_buffer(
                &pipeline.feedback_buffer,
                0,
                &pipeline.feedback_result_buffer,
                0,
                pipeline.feedback_result_buffer.size(),
            );
        }

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("render-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.01,
                        g: 0.01,
                        b: 0.01,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });

        render_pass.set_pipeline(&pipeline.render_pipeline);
        render_pass.set_bind_group(0, &pipeline.render_bind_group, &[]);

        render_pass.draw(0..3, 0..1);

        drop(render_pass);

        self.queue.submit(iter::once(encoder.finish()));

        if read_feedback {
            let (tx, rx) = mpsc::channel();

            pipeline.feedback_result_buffer.slice(..).map_async(
                wgpu::MapMode::Read,
                move |result| {
                    let _ = tx.send(result);
                },
            );

            self.state.feedback_rx = Some(rx);
        }

        output.present();
        self.window.request_redraw();
    }

    fn get_present_mode(vsync: bool) -> PresentMode {
        if vsync {
            PresentMode::AutoVsync
        } else {
            PresentMode::AutoNoVsync
        }
    }

    fn render_size(size: PhysicalSize<u32>, max_texture_dimension_2d: u32) -> PhysicalSize<u32> {
        if size.width == 0
            || size.height == 0
            || (size.width <= max_texture_dimension_2d && size.height <= max_texture_dimension_2d)
        {
            return size;
        }

        let scale = max_texture_dimension_2d as f64 / size.width.max(size.height) as f64;

        PhysicalSize {
            width: ((size.width as f64 * scale).floor() as u32).max(1),
            height: ((size.height as f64 * scale).floor() as u32).max(1),
        }
    }
}
