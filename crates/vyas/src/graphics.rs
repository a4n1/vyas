use std::{iter, sync::Arc};

use winit::{dpi::PhysicalSize, window::Window};

use crate::{
    buffer::{ChunkBuffer, ChunkBufferItem},
    chunk::{Chunk, ChunkMap},
    config::RenderConfig,
    ecs::World,
    pipeline::Pipeline,
};

pub(crate) struct Graphics {
    pub(crate) window: Arc<Window>,
    pub(crate) device: wgpu::Device,
    pub(crate) surface_config: wgpu::SurfaceConfiguration,
    surface: wgpu::Surface<'static>,
    queue: wgpu::Queue,
    state: State,
}

struct State {
    is_surface_configured: bool,
    chunk_buffer: ChunkBuffer,
    draw_chunks: Vec<ChunkBufferItem>,
}

impl Graphics {
    pub(crate) async fn new(window: Arc<Window>, render_config: &RenderConfig) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::BROWSER_WEBGPU,
            flags: Default::default(),
            memory_budget_thresholds: Default::default(),
            backend_options: Default::default(),
            display: None,
        });

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

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                #[cfg(not(target_arch = "wasm32"))]
                required_limits: wgpu::Limits::default(),
                #[cfg(target_arch = "wasm32")]
                required_limits: wgpu::Limits::downlevel_webgl2_defaults(),
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await
            .expect("failed to request device");

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format.remove_srgb_suffix(),
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            desired_maximum_frame_latency: 2,
            view_formats: vec![surface_format.add_srgb_suffix()],
        };

        Self {
            window,
            device,
            surface_config,
            surface,
            queue,
            state: State {
                is_surface_configured: false,
                chunk_buffer: ChunkBuffer::new(render_config),
                draw_chunks: Vec::new(),
            },
        }
    }

    pub(crate) fn resize(&mut self, PhysicalSize { width, height }: PhysicalSize<u32>) {
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface.configure(&self.device, &self.surface_config);
        self.state.is_surface_configured = true;
    }

    pub(crate) fn update(&mut self, pipeline: &Pipeline, world: &World) {
        let chunk_map = world.resource::<ChunkMap>();
        let render_config = *world.resource::<RenderConfig>();
        let chunks = chunk_map.visible_chunk_entities(world);

        self.state.draw_chunks = Vec::with_capacity(chunks.len());

        for entity in &chunks {
            let mut chunk = world
                .get_mut::<Chunk>(*entity)
                .expect("entity should have a chunk component");

            if let Some(buffer_chunk) = self.state.chunk_buffer.insert(
                entity,
                &mut chunk,
                pipeline,
                &self.queue,
                &render_config,
            ) {
                self.state.draw_chunks.push(buffer_chunk);
            }
        }

        self.state.chunk_buffer.cleanup();

        self.queue.write_buffer(
            &pipeline.camera_buffer,
            0,
            bytemuck::cast_slice(&[pipeline.camera_uniform]),
        );
    }

    pub(crate) fn render(&self, pipeline: &Pipeline) {
        if !self.state.is_surface_configured {
            return;
        }

        let output = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(surface_texture) => surface_texture,
            wgpu::CurrentSurfaceTexture::Suboptimal(surface_texture) => {
                self.surface.configure(&self.device, &self.surface_config);
                surface_texture
            }
            wgpu::CurrentSurfaceTexture::Timeout
            | wgpu::CurrentSurfaceTexture::Occluded
            | wgpu::CurrentSurfaceTexture::Validation => {
                return;
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.surface_config);
                return;
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                panic!("Lost device");
            }
        };

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(self.surface_config.format.add_srgb_suffix()),
            ..Default::default()
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &pipeline.depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(0.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });

        render_pass.set_pipeline(&pipeline.render_pipeline);
        render_pass.set_bind_group(0, &pipeline.camera_bind_group, &[]);

        for draw_chunk in &self.state.draw_chunks {
            render_pass.set_vertex_buffer(
                0,
                pipeline.vertex_buffer.slice(
                    draw_chunk.vertex_allocation.offset
                        ..draw_chunk.vertex_allocation.offset + draw_chunk.vertex_byte_len,
                ),
            );

            render_pass.set_index_buffer(
                pipeline.index_buffer.slice(
                    draw_chunk.index_allocation.offset
                        ..draw_chunk.index_allocation.offset + draw_chunk.index_byte_len,
                ),
                wgpu::IndexFormat::Uint32,
            );

            render_pass.draw_indexed(0..draw_chunk.mesh_index_count, 0, 0..1);
        }

        drop(render_pass);

        self.queue.submit(iter::once(encoder.finish()));
        output.present();
        self.window.request_redraw();
    }
}
