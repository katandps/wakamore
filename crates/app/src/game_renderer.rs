use std::{collections::HashMap, time::Instant};
use wgpu::util::DeviceExt;
use winit::window::Window;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    texture_coordinates: [f32; 2],
    opacity: f32,
}

#[derive(Clone, Copy)]
pub struct Rect<T> {
    pub position: [T; 2],
    pub size: [T; 2],
}

pub struct ImageDrawOptions<'a> {
    pub path: &'a str,
    pub source: Rect<u32>,
    pub destination: Rect<f32>,
    pub opacity: f32,
}

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

const VERTEX_COUNT: u32 = 6;
fn image_vertices(
    options: &ImageDrawOptions,
    texture_size: [u32; 2],
) -> [Vertex; VERTEX_COUNT as usize] {
    let source_left = options.source.position[0] as f32 / texture_size[0] as f32;
    let source_top = options.source.position[1] as f32 / texture_size[1] as f32;
    let source_right =
        (options.source.position[0] + options.source.size[0]) as f32 / texture_size[0] as f32;
    let source_bottom =
        (options.source.position[1] + options.source.size[1]) as f32 / texture_size[1] as f32;
    let left = options.destination.position[0];
    let top = options.destination.position[1];
    let right = left + options.destination.size[0];
    let bottom = top - options.destination.size[1];
    let opacity = options.opacity.clamp(0.0, 1.0);

    [
        Vertex {
            position: [left, top],
            texture_coordinates: [source_left, source_top],
            opacity,
        },
        Vertex {
            position: [right, top],
            texture_coordinates: [source_right, source_top],
            opacity,
        },
        Vertex {
            position: [left, bottom],
            texture_coordinates: [source_left, source_bottom],
            opacity,
        },
        Vertex {
            position: [right, top],
            texture_coordinates: [source_right, source_top],
            opacity,
        },
        Vertex {
            position: [right, bottom],
            texture_coordinates: [source_right, source_bottom],
            opacity,
        },
        Vertex {
            position: [left, bottom],
            texture_coordinates: [source_left, source_bottom],
            opacity,
        },
    ]
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
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    images: HashMap<String, ImageTexture>,
    started_at: Instant,
}

#[derive(Clone, Copy)]
pub enum GameRenderState {
    Title,
    Gameplay,
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
        let initial_options = ImageDrawOptions {
            path: "resources/circles.png",
            source: Rect {
                position: [0, 0],
                size: [1, 1],
            },
            destination: Rect {
                position: [-0.275, 0.275],
                size: [0.55, 0.55],
            },
            opacity: 1.0,
        };
        let initial_texture = load_image_texture(
            &device,
            &queue,
            &texture_bind_group_layout,
            initial_options.path,
        );
        let initial_options = ImageDrawOptions {
            source: Rect {
                size: initial_texture.size,
                ..initial_options.source
            },
            ..initial_options
        };
        let initial_vertices = image_vertices(&initial_options, initial_texture.size);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("2D rectangle shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("2D pipeline layout"),
            bind_group_layouts: &[Some(&texture_bind_group_layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("2D texture pipeline"),
            layout: Some(&pipeline_layout),
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
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("animated sprite vertices"),
            contents: bytemuck::cast_slice(&initial_vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            surface,
            device,
            queue,
            config,
            pipeline,
            vertex_buffer,
            texture_bind_group_layout,
            images: HashMap::from([(initial_options.path.to_owned(), initial_texture)]),
            started_at: Instant::now(),
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

    pub fn render(&mut self, state: GameRenderState) -> bool {
        let elapsed = self.started_at.elapsed().as_secs_f32();
        let options = ImageDrawOptions {
            path: "resources/circles.png",
            source: Rect {
                position: [0, 0],
                size: [80, 80],
            },
            destination: Rect {
                position: [
                    0.70 * (elapsed * 0.55).sin() - 0.275,
                    0.42 * (elapsed * 0.80).sin() + 0.275,
                ],
                size: [0.55, 0.55],
            },
            opacity: 0.20 + 0.80 * (0.5 + 0.5 * (elapsed * 1.20).sin()),
        };
        let texture_size = self.image_texture(options.path).size;
        let vertices = image_vertices(&options, texture_size);
        self.queue
            .write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame)
            | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(&self.device, &self.config);
                return false;
            }
            wgpu::CurrentSurfaceTexture::Timeout
            | wgpu::CurrentSurfaceTexture::Occluded
            | wgpu::CurrentSurfaceTexture::Validation => return false,
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("2D render encoder"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("2D render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(match state {
                            GameRenderState::Title => wgpu::Color {
                                r: 0.03,
                                g: 0.05,
                                b: 0.10,
                                a: 1.0,
                            },
                            GameRenderState::Gameplay => wgpu::Color {
                                r: 0.02,
                                g: 0.12,
                                b: 0.08,
                                a: 1.0,
                            },
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                multiview_mask: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(
                0,
                &self
                    .images
                    .get(options.path)
                    .expect("画像テクスチャのキャッシュ取得に失敗しました")
                    .bind_group,
                &[],
            );
            pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            pass.draw(0..VERTEX_COUNT, 0..1);
        }
        self.queue.submit(Some(encoder.finish()));
        self.queue.present(frame);
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
