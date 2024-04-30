// TODO: consider MDI (multi draw indirect)

use log::{debug, info, trace};
use wgpu::util::DeviceExt;
use winit::{dpi::PhysicalSize, window::Window};

pub mod cached;
pub mod camera;
pub mod light;
pub mod mesh;
pub mod texture;
pub mod vertex;

use cached::CachedMesh;
use camera::{Camera, CameraUniform};
use light::LightUniform;
use mesh::Mesh;
use texture::Texture;
use vertex::Vertex;

pub struct Renderer {
    _instance: wgpu::Instance,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    pub size: PhysicalSize<u32>,

    pipelines: [wgpu::RenderPipeline; 2],
    depth_texture: Texture,

    sun: wgpu::Buffer,
    sun_bind_group: wgpu::BindGroup,

    pub camera: Camera,
    camera_buffer: wgpu::Buffer,
    camera_uniform: CameraUniform,
    camera_bind_group: wgpu::BindGroup,
}

impl Renderer {
    pub async fn new(window: &'static Window) -> Self {
        let size = window.inner_size();

        let backends = wgpu::Backends::PRIMARY;
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends,
            ..Default::default()
        });
        debug!("Instance created");

        let surface = instance
            .create_surface(window)
            .expect("failed to create surface");
        debug!("Surface created");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::None,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("failed to get adapter");
        debug!("Adapter acquired");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Device"),
                    required_features: wgpu::Features::POLYGON_MODE_LINE,
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .expect("failed to get device");
        debug!("Device and queue acquired");

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|s| s.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 2, // max images in flight for swapchain
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);
        debug!("Surface configured");

        let depth_texture = Texture::create_depth_texture(&device, &config, "Depth texture");
        debug!("Depth texture created");

        let eye = (0.0, 1.0, 0.0).into();
        let camera = Camera {
            eye,
            target: eye + vek::Vec3::new(0.0, 0.0, -1.0),
            up: vek::Vec3::unit_y(),
            aspect: config.width as f32 / config.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 1000.0,
        };

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("Camera Bind Group Layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });
        debug!("Camera initialized");

        let light_uniform = LightUniform::new([0.0, 0.0, 0.0], [0.0, 0.0, 0.0]);

        let sun = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light VB"),
            contents: bytemuck::cast_slice(&[light_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let sun_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: None,
            });

        let sun_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &sun_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: sun.as_entire_binding(),
            }],
            label: None,
        });
        debug!("Sun initialized");

        let vert_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Vertex Shader"),
            source: wgpu::ShaderSource::Wgsl(
                concat!(
                    include_str!("shaders/common.wgsl"),
                    include_str!("shaders/vert.wgsl")
                )
                .into(),
            ),
        });

        let frag_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Fragment Shader"),
            source: wgpu::ShaderSource::Wgsl(
                concat!(
                    include_str!("shaders/common.wgsl"),
                    include_str!("shaders/frag.wgsl")
                )
                .into(),
            ),
        });

        debug!("Shaders initialized");

        let fill_render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Fill render"),
                bind_group_layouts: &[&camera_bind_group_layout, &sun_bind_group_layout],
                push_constant_ranges: &[],
            });

        let fill_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&fill_render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vert_shader,
                entry_point: "main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &frag_shader,
                entry_point: "main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,

                    blend: None,
                    // blend: Some(wgpu::BlendState {
                    // color: wgpu::BlendComponent {
                    // src_factor: wgpu::BlendFactor::SrcAlpha,
                    // dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    // operation: wgpu::BlendOperation::Add,
                    // },
                    // alpha: wgpu::BlendComponent::OVER,
                    // }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let wire_render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Wire render"),
                bind_group_layouts: &[&camera_bind_group_layout, &sun_bind_group_layout],
                push_constant_ranges: &[],
            });

        let wire_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&wire_render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vert_shader,
                entry_point: "main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &frag_shader,
                entry_point: "main",
                targets: &[Some(surface_format.into())],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Line,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });
        debug!("Pipeline initialized");

        info!("Renderer initialized");
        Self {
            _instance: instance,
            device,
            queue,
            surface,
            config,
            size,

            pipelines: [fill_render_pipeline, wire_render_pipeline],
            depth_texture,

            sun,
            sun_bind_group,

            camera,
            camera_buffer,
            camera_uniform,
            camera_bind_group,
        }
    }

    pub fn next_pipeline(&mut self) {
        let [l, r] = &mut self.pipelines;
        std::mem::swap(l, r);
    }

    pub fn light(&mut self, light: LightUniform) {
        self.queue
            .write_buffer(&self.sun, 0, bytemuck::cast_slice(&[light]));
        trace!("Updated light uniform");
    }

    pub fn cache(&mut self, mesh: Mesh) -> CachedMesh {
        CachedMesh::new(mesh, &mut self.device)
    }

    pub fn update_cache(&mut self, cache: &mut CachedMesh, mesh: Mesh) {
        cache.update(mesh, &mut self.device, &mut self.queue)
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.camera.aspect = new_size.width as f32 / new_size.height as f32;
            self.depth_texture =
                Texture::create_depth_texture(&self.device, &self.config, "Depth texture");
        }
    }

    pub fn render<'a>(
        &mut self,
        meshes: impl IntoIterator<Item = &'a CachedMesh>,
    ) -> anyhow::Result<()> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.pipelines[0]);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_bind_group(1, &self.sun_bind_group, &[]);

            for mesh in meshes {
                render_pass.set_vertex_buffer(0, mesh.vertices.slice(..));
                render_pass.set_index_buffer(mesh.indices.slice(..), wgpu::IndexFormat::Uint32);

                render_pass.draw_indexed(0..mesh.num_indices, 0, 0..1);
                trace!("Making render pass");
            }
        }

        self.queue.submit([encoder.finish()]);
        output.present();
        trace!("Queue flushed");
        Ok(())
    }

    pub fn update_camera(
        &mut self,
        controller: &mut crate::app::CameraController,
        dt: std::time::Duration,
    ) {
        controller.update_camera(&mut self.camera, dt);
        self.camera_uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }
}
