use std::collections::HashMap;
use wgpu::util::DeviceExt;
use winit::window::Window;

use crate::renderer::image::{IMAGE_VERTEX_COUNT, ImageDrawOptions, ImageVertex};
use crate::renderer::rectangle::{RECTANGLE_VERTEX_COUNT, RectangleDrawOptions, RectangleVertex};

type Vertex = ImageVertex;

impl Vertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Float32];

    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

impl RectangleVertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x4];

    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

struct ImageTexture {
    bind_group: wgpu::BindGroup,
    size: [u32; 2],
}

pub struct GameRenderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    image_pipeline: wgpu::RenderPipeline,
    rectangle_pipeline: wgpu::RenderPipeline,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    images: HashMap<String, ImageTexture>,
}

enum DrawCommand {
    Image {
        path: String,
        vertices: [ImageVertex; IMAGE_VERTEX_COUNT],
    },
    Rectangle([RectangleVertex; RECTANGLE_VERTEX_COUNT]),
}

pub struct GameFrame<'a> {
    renderer: &'a mut GameRenderer,
    frame: wgpu::SurfaceTexture,
    view: wgpu::TextureView,
    encoder: wgpu::CommandEncoder,
    commands: Vec<DrawCommand>,
}

impl GameRenderer {
    pub async fn new(window: &'static Window) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let surface = instance
            .create_surface(window)
            .expect("ゲーム Surface の作成に失敗しました");
        let adapter = request_adapter(&instance, &surface).await;
        let (device, queue) = request_device(&adapter).await;
        let config = configure_surface(&surface, &adapter, &device, size);
        let format = config.format;

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("image texture bind group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("2D rectangle shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        let image_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("2D pipeline layout"),
                bind_group_layouts: &[Some(&texture_bind_group_layout)],
                immediate_size: 0,
            });
        let image_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("2D texture pipeline"),
            layout: Some(&image_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(Vertex::layout())],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        let rectangle_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("rectangle pipeline layout"),
                bind_group_layouts: &[],
                immediate_size: 0,
            });
        let rectangle_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("rectangle pipeline"),
            layout: Some(&rectangle_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_rectangle"),
                buffers: &[Some(RectangleVertex::layout())],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_rectangle"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        Self {
            surface,
            device,
            queue,
            config,
            image_pipeline,
            rectangle_pipeline,
            texture_bind_group_layout,
            images: HashMap::new(),
        }
    }

    pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            return;
        }
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
    }

    fn image_texture(&mut self, path: &str) -> &ImageTexture {
        if !self.images.contains_key(path) {
            let image_texture = load_image_texture(
                &self.device,
                &self.queue,
                &self.texture_bind_group_layout,
                path,
            );
            self.images.insert(path.to_owned(), image_texture);
        }
        self.images
            .get(path)
            .expect("画像テクスチャのキャッシュ取得に失敗しました")
    }

    pub fn begin_frame(&mut self) -> Option<GameFrame<'_>> {
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame)
            | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(&self.device, &self.config);
                return None;
            }
            wgpu::CurrentSurfaceTexture::Timeout
            | wgpu::CurrentSurfaceTexture::Occluded
            | wgpu::CurrentSurfaceTexture::Validation => return None,
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("2D render encoder"),
            });

        Some(GameFrame {
            renderer: self,
            frame,
            view,
            encoder,
            commands: Vec::new(),
        })
    }
}

impl GameFrame<'_> {
    pub fn draw_image(&mut self, options: &ImageDrawOptions<'_>) {
        let texture_size = self.renderer.image_texture(options.src_path()).size;
        let vertices = options.vertices(texture_size);
        self.commands.push(DrawCommand::Image {
            path: options.src_path().to_owned(),
            vertices,
        });
    }

    pub fn draw_rectangle(&mut self, options: &RectangleDrawOptions) {
        self.commands
            .push(DrawCommand::Rectangle(options.vertices()));
    }

    pub fn end_frame(self) -> bool {
        let GameFrame {
            renderer,
            frame,
            view,
            mut encoder,
            commands,
        } = self;
        let image_vertices: Vec<ImageVertex> = commands
            .iter()
            .filter_map(|command| match command {
                DrawCommand::Image { vertices, .. } => Some(vertices),
                DrawCommand::Rectangle(_) => None,
            })
            .flat_map(|vertices| vertices.iter().copied())
            .collect();
        let rectangle_vertices: Vec<RectangleVertex> = commands
            .iter()
            .filter_map(|command| match command {
                DrawCommand::Image { .. } => None,
                DrawCommand::Rectangle(vertices) => Some(vertices),
            })
            .flat_map(|vertices| vertices.iter().copied())
            .collect();
        let image_vertex_buffer = (!image_vertices.is_empty()).then(|| {
            renderer
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("image vertices"),
                    contents: bytemuck::cast_slice(&image_vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                })
        });
        let rectangle_vertex_buffer = (!rectangle_vertices.is_empty()).then(|| {
            renderer
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("rectangle vertices"),
                    contents: bytemuck::cast_slice(&rectangle_vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                })
        });
        let mut image_index = 0;
        let mut rectangle_index = 0;

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("2D render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                multiview_mask: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            for command in &commands {
                match command {
                    DrawCommand::Image { path, .. } => {
                        pass.set_pipeline(&renderer.image_pipeline);
                        pass.set_bind_group(
                            0,
                            &renderer
                                .images
                                .get(path)
                                .expect("画像テクスチャのキャッシュ取得に失敗しました")
                                .bind_group,
                            &[],
                        );
                        let offset =
                            (image_index * IMAGE_VERTEX_COUNT * std::mem::size_of::<ImageVertex>())
                                as wgpu::BufferAddress;
                        pass.set_vertex_buffer(
                            0,
                            image_vertex_buffer
                                .as_ref()
                                .expect("画像頂点バッファがありません")
                                .slice(offset..),
                        );
                        pass.draw(0..IMAGE_VERTEX_COUNT as u32, 0..1);
                        image_index += 1;
                    }
                    DrawCommand::Rectangle(_) => {
                        pass.set_pipeline(&renderer.rectangle_pipeline);
                        let offset = (rectangle_index
                            * RECTANGLE_VERTEX_COUNT
                            * std::mem::size_of::<RectangleVertex>())
                            as wgpu::BufferAddress;
                        pass.set_vertex_buffer(
                            0,
                            rectangle_vertex_buffer
                                .as_ref()
                                .expect("矩形頂点バッファがありません")
                                .slice(offset..),
                        );
                        pass.draw(0..RECTANGLE_VERTEX_COUNT as u32, 0..1);
                        rectangle_index += 1;
                    }
                }
            }
        }
        renderer.queue.submit(Some(encoder.finish()));
        renderer.queue.present(frame);
        true
    }
}

fn load_image_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    bind_group_layout: &wgpu::BindGroupLayout,
    path: &str,
) -> ImageTexture {
    let image = image::open(path)
        .unwrap_or_else(|error| panic!("画像の読み込みに失敗しました ({path}): {error}"))
        .to_rgba8();
    let size = [image.width(), image.height()];
    let image_size = wgpu::Extent3d {
        width: size[0],
        height: size[1],
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("image texture"),
        size: image_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &image,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * size[0]),
            rows_per_image: Some(size[1]),
        },
        image_size,
    );
    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("image sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("image texture bind group"),
        layout: bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });

    ImageTexture { bind_group, size }
}

async fn request_adapter(instance: &wgpu::Instance, surface: &wgpu::Surface<'_>) -> wgpu::Adapter {
    instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(surface),
            force_fallback_adapter: false,
            apply_limit_buckets: false,
        })
        .await
        .expect("利用可能な GPU アダプターがありません")
}

async fn request_device(adapter: &wgpu::Adapter) -> (wgpu::Device, wgpu::Queue) {
    adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("wgpu device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            memory_hints: wgpu::MemoryHints::Performance,
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
            trace: wgpu::Trace::Off,
        })
        .await
        .expect("GPU デバイスの作成に失敗しました")
}

fn configure_surface(
    surface: &wgpu::Surface<'_>,
    adapter: &wgpu::Adapter,
    device: &wgpu::Device,
    size: winit::dpi::PhysicalSize<u32>,
) -> wgpu::SurfaceConfiguration {
    let capabilities = surface.get_capabilities(adapter);
    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: capabilities.formats[0],
        width: size.width.max(1),
        height: size.height.max(1),
        present_mode: wgpu::PresentMode::Immediate,
        alpha_mode: capabilities.alpha_modes[0],
        view_formats: vec![],
        color_space: wgpu::SurfaceColorSpace::Srgb,
        desired_maximum_frame_latency: 2,
    };
    surface.configure(device, &config);
    config
}
