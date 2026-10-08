use bytemuck::Zeroable;
use wgpu::{Limits, util::DeviceExt};

use crate::{
    brickgrid::FeedbackGpu,
    camera::{CameraGpu, CameraState},
    color::ColorPaletteState,
    config::{RenderConfigState, Resolution},
    ecs::World,
    graphics::Graphics,
    light::LightsGpu,
};

pub struct Pipeline {
    pub(crate) camera: CameraGpu,
    pub(crate) camera_buffer: wgpu::Buffer,
    pub(crate) render_config: RenderConfigState,
    pub(crate) render_config_buffer: wgpu::Buffer,
    pub(crate) lights_buffer: wgpu::Buffer,
    pub(crate) color_palette_buffer: wgpu::Buffer,
    scene_bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) scene_bind_group: wgpu::BindGroup,
    render_bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) render_bind_group: wgpu::BindGroup,
    base_color_sampler: wgpu::Sampler,
    base_color_texture: wgpu::Texture,
    surface_data_texture: wgpu::Texture,
    pub(crate) compute_pipeline: wgpu::ComputePipeline,
    pub(crate) render_pipeline: wgpu::RenderPipeline,
    pub(crate) brickgrid_buffer: wgpu::Buffer,
    pub(crate) brickmap_buffer: wgpu::Buffer,
    pub(crate) color_buffer: wgpu::Buffer,
    pub(crate) feedback_buffer: wgpu::Buffer,
    pub(crate) feedback_result_buffer: wgpu::Buffer,
}

impl Pipeline {
    pub(crate) fn new(graphics: &Graphics, world: &World) -> Self {
        let shader = graphics
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("./shaders/shader.wgsl").into()),
            });

        let camera = world.resource::<CameraState>();
        let mut camera_gpu = CameraGpu::new();
        camera_gpu.update_view_proj(&camera);

        let camera_buffer = graphics
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("camera-buffer"),
                contents: bytemuck::cast_slice(&[camera_gpu]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let render_config = *world.resource::<RenderConfigState>();

        let color_palette = world.resource::<ColorPaletteState>();

        let render_config_buffer =
            graphics
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("render-config-buffer"),
                    contents: bytemuck::cast_slice(&[render_config.to_gpu()]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        let color_palette_buffer =
            graphics
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("color-palette-buffer"),
                    contents: bytemuck::cast_slice(&color_palette.to_gpu()),
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                });

        let brickgrid_buffer = graphics.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("brickgrid-buffer"),
            size: render_config.brickgrid_size.pow(3) as u64 * 4,
            mapped_at_creation: false,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let feedback_buffer = graphics.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("feedback-buffer"),
            size: size_of::<FeedbackGpu>() as u64,
            mapped_at_creation: false,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
        });

        let feedback_result_buffer = graphics.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("feedback-result-buffer"),
            size: size_of::<FeedbackGpu>() as u64,
            mapped_at_creation: false,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        });

        let brickmap_buffer = graphics.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("brickmap-buffer"),
            size: Limits::default().max_storage_buffer_binding_size,
            mapped_at_creation: false,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let color_buffer = graphics.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("color-buffer"),
            size: Limits::default().max_storage_buffer_binding_size,
            mapped_at_creation: false,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let lights_buffer = graphics
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("lights-buffer"),
                contents: bytemuck::bytes_of(&LightsGpu::zeroed()),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let base_color_texture = Self::create_base_color_texture(graphics, render_config.resolution);
        let base_color_view = base_color_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let surface_data_texture = Self::create_surface_data_texture(graphics, base_color_texture.size());
        let surface_data_view = surface_data_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let base_color_sampler = graphics.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("base-color-sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let scene_bind_group_layout =
            graphics
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 3,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: false },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 4,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: false },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 5,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 6,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 7,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 8,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::StorageTexture {
                                access: wgpu::StorageTextureAccess::WriteOnly,
                                format: wgpu::TextureFormat::Rgba16Float,
                                view_dimension: wgpu::TextureViewDimension::D2,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 10,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::StorageTexture {
                                access: wgpu::StorageTextureAccess::WriteOnly,
                                format: wgpu::TextureFormat::Rgba8Uint,
                                view_dimension: wgpu::TextureViewDimension::D2,
                            },
                            count: None,
                        },
                    ],
                    label: Some("scene-bind-group-layout"),
                });

        let scene_bind_group = graphics
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &scene_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: camera_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: render_config_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: color_palette_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: brickgrid_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: feedback_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: brickmap_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 6,
                        resource: color_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 7,
                        resource: lights_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 8,
                        resource: wgpu::BindingResource::TextureView(&base_color_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 10,
                        resource: wgpu::BindingResource::TextureView(&surface_data_view),
                    },
                ],
                label: Some("scene-bind-group"),
            });

        let compute_pipeline_layout =
            graphics
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("compute-pipeline-layout"),
                    bind_group_layouts: &[Some(&scene_bind_group_layout)],
                    immediate_size: 0,
                });

        let compute_pipeline =
            graphics
                .device
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("compute-pipeline"),
                    layout: Some(&compute_pipeline_layout),
                    module: &shader,
                    entry_point: Some("raytrace"),
                    compilation_options: Default::default(),
                    cache: None,
                });

        let render_bind_group_layout =
            graphics
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 7,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 9,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 11,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Uint,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 12,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                    label: Some("render-bind-group-layout"),
                });

        let render_bind_group = graphics
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &render_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 7,
                        resource: lights_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 9,
                        resource: wgpu::BindingResource::TextureView(&base_color_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 11,
                        resource: wgpu::BindingResource::TextureView(&surface_data_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 12,
                        resource: wgpu::BindingResource::Sampler(&base_color_sampler),
                    },
                ],
                label: Some("render-bind-group"),
            });

        let render_pipeline_layout =
            graphics
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("render-pipeline-layout"),
                    bind_group_layouts: &[Some(&render_bind_group_layout)],
                    immediate_size: 0,
                });

        let render_pipeline =
            graphics
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("render-pipeline"),
                    layout: Some(&render_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: Some("vs_main"),
                        buffers: &[],
                        compilation_options: Default::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: Some("fs_main"),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: graphics.render_format(),
                            blend: Some(wgpu::BlendState {
                                color: wgpu::BlendComponent::REPLACE,
                                alpha: wgpu::BlendComponent::REPLACE,
                            }),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                        compilation_options: Default::default(),
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: None,
                        polygon_mode: wgpu::PolygonMode::Fill,
                        unclipped_depth: false,
                        conservative: false,
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState::default(),
                    multiview_mask: None,
                    cache: None,
                });

        Self {
            camera: camera_gpu,
            camera_buffer,
            render_config,
            render_config_buffer,
            lights_buffer,
            color_palette_buffer,
            scene_bind_group_layout,
            scene_bind_group,
            render_bind_group_layout,
            render_bind_group,
            base_color_sampler,
            base_color_texture,
            surface_data_texture,
            compute_pipeline,
            render_pipeline,
            brickgrid_buffer,
            brickmap_buffer,
            color_buffer,
            feedback_buffer,
            feedback_result_buffer,
        }
    }

    pub(crate) fn output_size(&self) -> wgpu::Extent3d {
        self.base_color_texture.size()
    }

    pub(crate) fn update(&mut self, graphics: &Graphics, world: &World) {
        let camera = world.resource::<CameraState>();
        self.camera.update_view_proj(&camera);

        let render_config = world.resource::<RenderConfigState>();

        let mut is_change = false;

        if self.render_config.resolution != render_config.resolution {
            is_change = true;
        }

        if self.render_config.brickgrid_size != render_config.brickgrid_size {
            is_change = true;

            self.brickgrid_buffer = graphics.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("brickgrid-buffer"),
                size: render_config.brickgrid_size.pow(3) as u64 * 4,
                mapped_at_creation: false,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });
        }

        self.render_config = *render_config;

        if is_change {
            self.resize(graphics);
        }
    }

    pub(crate) fn resize(&mut self, graphics: &Graphics) {
        let base_color_texture = Self::create_base_color_texture(graphics, self.render_config.resolution);
        let base_color_view = base_color_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let surface_data_texture = Self::create_surface_data_texture(graphics, base_color_texture.size());
        let surface_data_view = surface_data_texture.create_view(&wgpu::TextureViewDescriptor::default());

        self.rebuild_scene_bind_group(graphics, &base_color_view, &surface_data_view);

        self.render_bind_group = graphics
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.render_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 7,
                        resource: self.lights_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 9,
                        resource: wgpu::BindingResource::TextureView(&base_color_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 11,
                        resource: wgpu::BindingResource::TextureView(&surface_data_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 12,
                        resource: wgpu::BindingResource::Sampler(&self.base_color_sampler),
                    },
                ],
                label: Some("render-bind-group"),
            });

        self.base_color_texture = base_color_texture;
        self.surface_data_texture = surface_data_texture;
    }

    fn rebuild_scene_bind_group(
        &mut self,
        graphics: &Graphics,
        base_color_view: &wgpu::TextureView,
        surface_data_view: &wgpu::TextureView,
    ) {
        self.scene_bind_group = graphics
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.scene_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: self.camera_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: self.render_config_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: self.color_palette_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: self.brickgrid_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: self.feedback_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: self.brickmap_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 6,
                        resource: self.color_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 7,
                        resource: self.lights_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 8,
                        resource: wgpu::BindingResource::TextureView(base_color_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 10,
                        resource: wgpu::BindingResource::TextureView(surface_data_view),
                    },
                ],
                label: Some("scene-bind-group"),
            });
    }

    fn create_base_color_texture(graphics: &Graphics, resolution: Resolution) -> wgpu::Texture {
        let (width, height) = match resolution {
            Resolution::Scale(scale) => {
                let width = ((graphics.surface_config.width as f64 * scale).floor() as u32).max(1);
                let height =
                    ((graphics.surface_config.height as f64 * scale).floor() as u32).max(1);

                (width, height)
            }
            Resolution::Dimensions { width, height } => (width, height),
        };

        graphics.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("base-color-texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        })
    }

    fn create_surface_data_texture(graphics: &Graphics, size: wgpu::Extent3d) -> wgpu::Texture {
        graphics.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("surface-data-texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Uint,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        })
    }
}
