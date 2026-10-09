use egui::{Context, ViewportId};
use egui_wgpu::{Renderer, ScreenDescriptor};
use egui_winit::State;
use winit::window::Window;

pub(crate) struct Egui {
    pub(crate) state: State,
    pub(crate) renderer: Renderer,
    pub(crate) ui: Option<Box<dyn FnMut(&::egui::Context)>>,
}

impl Egui {
    pub(crate) fn new(
        window: &Window,
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        max_texture_side: u32,
        ui: Option<Box<dyn FnMut(&Context)>>,
    ) -> Self {
        let context = Context::default();
        let state = State::new(
            context,
            ViewportId::ROOT,
            window,
            Some(window.scale_factor() as f32),
            None,
            Some(max_texture_side as usize),
        );
        let renderer = egui_wgpu::Renderer::new(device, surface_format, Default::default());

        Self {
            state,
            renderer,
            ui,
        }
    }

    pub(crate) fn render(
        &mut self,
        window: &Window,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        mut encoder: wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        size: [u32; 2],
    ) {
        let raw_input = self.state.take_egui_input(window);
        let context = self.state.egui_ctx().clone();
        let full_output = context.run_ui(raw_input, |root_ui| {
            if let Some(ui) = &mut self.ui {
                ui(root_ui.ctx());
            }
        });

        self.state
            .handle_platform_output(window, full_output.platform_output);

        let paint_jobs = context.tessellate(full_output.shapes, full_output.pixels_per_point);
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: size,
            pixels_per_point: full_output.pixels_per_point,
        };

        for (id, image_delta) in &full_output.textures_delta.set {
            self.renderer
                .update_texture(device, queue, *id, image_delta);
        }

        let command_buffers = self.renderer.update_buffers(
            device,
            queue,
            &mut encoder,
            &paint_jobs,
            &screen_descriptor,
        );

        let mut pass = encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            })
            .forget_lifetime();

        self.renderer
            .render(&mut pass, &paint_jobs, &screen_descriptor);

        drop(pass);

        queue.submit(
            command_buffers
                .into_iter()
                .chain(std::iter::once(encoder.finish())),
        );

        for id in &full_output.textures_delta.free {
            self.renderer.free_texture(id);
        }
    }
}
