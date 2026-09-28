pub mod configure;
pub mod image;
pub mod rectangle;

use std::collections::HashMap;
use wgpu::util::DeviceExt;
use winit::window::Window;

use crate::renderer::image::{IMAGE_VERTEX_COUNT, ImageDrawOptions, ImageTexture, ImageVertex};
use crate::renderer::rectangle::{RECTANGLE_VERTEX_COUNT, RectangleDrawOptions, RectangleVertex};

type Vertex = ImageVertex;

#[derive(Clone, Copy)]
pub struct Rect<T> {
    position: [T; 2],
    size: [T; 2],
}

impl<T: Copy> Rect<T> {
    pub const fn new(x: T, y: T, width: T, height: T) -> Self {
        Self {
            position: [x, y],
            size: [width, height],
        }
    }

    pub fn x(&self) -> T {
        self.position[0]
    }
    pub fn y(&self) -> T {
        self.position[1]
    }
    pub fn width(&self) -> T {
        self.size[0]
    }
    pub fn height(&self) -> T {
        self.size[1]
    }
}

#[derive(Clone, Copy)]
pub struct Color {
    red: f32,
    green: f32,
    blue: f32,
    alpha: f32,
}

impl Color {
    pub const fn new(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }
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
        let adapter = configure::request_adapter(&instance, &surface).await;
        let (device, queue) = configure::request_device(&adapter).await;
        let config = configure::configure_surface(&surface, &adapter, &device, size);
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
        let image_pipeline =
            configure::image_pipeline(&device, &shader, &config, &texture_bind_group_layout);
        let rectangle_pipeline = configure::rectangle_image_pipeline(&device, &shader, &config);
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
            let image_texture = image::load_image_texture(
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

    pub fn end_frame(mut self) -> bool {
        let mut image_vertices = Vec::new();
        let mut rectangle_vertices = Vec::new();

        for command in &self.commands {
            match command {
                DrawCommand::Image { vertices, .. } => {
                    image_vertices.extend_from_slice(vertices);
                }
                DrawCommand::Rectangle(vertices) => {
                    rectangle_vertices.extend_from_slice(vertices);
                }
            }
        }
        let image_vertex_buffer = (!image_vertices.is_empty()).then(|| {
            self.renderer
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("image vertices"),
                    contents: bytemuck::cast_slice(&image_vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                })
        });
        let rectangle_vertex_buffer = (!rectangle_vertices.is_empty()).then(|| {
            self.renderer
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
            let mut pass = self.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("2D render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.view,
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
            for command in &self.commands {
                match command {
                    DrawCommand::Image { path, .. } => {
                        Self::apply_draw_image(
                            path,
                            &mut pass,
                            self.renderer,
                            image_vertex_buffer
                                .as_ref()
                                .expect("画像頂点バッファが見つかりませんでした"),
                            image_index,
                        );
                        image_index += 1;
                    }
                    DrawCommand::Rectangle(_) => {
                        Self::apply_draw_rectangle(
                            &mut pass,
                            self.renderer,
                            rectangle_vertex_buffer
                                .as_ref()
                                .expect("矩形頂点バッファが見つかりませんでした"),
                            rectangle_index,
                        );
                        rectangle_index += 1;
                    }
                }
            }
        }
        self.renderer.queue.submit(Some(self.encoder.finish()));
        self.renderer.queue.present(self.frame);
        true
    }

    fn apply_draw_image(
        path: &str,
        pass: &mut wgpu::RenderPass<'_>,
        renderer: &GameRenderer,
        image_vertex_buffer: &wgpu::Buffer,
        image_index: usize,
    ) {
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
        let offset = (image_index * IMAGE_VERTEX_COUNT * std::mem::size_of::<ImageVertex>())
            as wgpu::BufferAddress;
        pass.set_vertex_buffer(0, image_vertex_buffer.slice(offset..));
        pass.draw(0..IMAGE_VERTEX_COUNT as u32, 0..1);
    }

    fn apply_draw_rectangle(
        pass: &mut wgpu::RenderPass<'_>,
        renderer: &GameRenderer,
        rectangle_vertex_buffer: &wgpu::Buffer,
        rectangle_index: usize,
    ) {
        pass.set_pipeline(&renderer.rectangle_pipeline);
        let offset = (rectangle_index
            * RECTANGLE_VERTEX_COUNT
            * std::mem::size_of::<RectangleVertex>()) as wgpu::BufferAddress;
        pass.set_vertex_buffer(0, rectangle_vertex_buffer.slice(offset..));
        pass.draw(0..RECTANGLE_VERTEX_COUNT as u32, 0..1);
    }
}
