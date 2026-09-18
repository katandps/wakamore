use wgpu::util::DeviceExt;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 3],
}

impl Vertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x3];

    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.85, 0.55],
        color: [0.15, 0.75, 0.95],
    },
    Vertex {
        position: [-0.15, 0.55],
        color: [0.15, 0.75, 0.95],
    },
    Vertex {
        position: [-0.85, -0.55],
        color: [0.15, 0.75, 0.95],
    },
    Vertex {
        position: [-0.15, 0.55],
        color: [0.15, 0.75, 0.95],
    },
    Vertex {
        position: [-0.15, -0.55],
        color: [0.15, 0.75, 0.95],
    },
    Vertex {
        position: [-0.85, -0.55],
        color: [0.15, 0.75, 0.95],
    },
    Vertex {
        position: [0.05, 0.82],
        color: [1.0, 0.42, 0.20],
    },
    Vertex {
        position: [0.80, 0.25],
        color: [1.0, 0.42, 0.20],
    },
    Vertex {
        position: [0.05, 0.25],
        color: [1.0, 0.42, 0.20],
    },
    Vertex {
        position: [0.05, 0.82],
        color: [1.0, 0.42, 0.20],
    },
    Vertex {
        position: [0.80, 0.82],
        color: [1.0, 0.42, 0.20],
    },
    Vertex {
        position: [0.80, 0.25],
        color: [1.0, 0.42, 0.20],
    },
];

struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
}

impl Renderer {
    async fn new(window: &'static Window) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let surface = instance
            .create_surface(window)
            .expect("Surface の作成に失敗しました");
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("利用可能な GPU アダプターがありません");
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("wgpu 2D demo device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            })
            .await
            .expect("GPU デバイスの作成に失敗しました");

        let capabilities = surface.get_capabilities(&adapter);
        let format = capabilities.formats[0];
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("2D rectangle shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("2D pipeline layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("2D rectangle pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::layout()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("2D rectangle vertices"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {
            surface,
            device,
            queue,
            config,
            pipeline,
            vertex_buffer,
        }
    }

    fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            return;
        }
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let frame = self.surface.get_current_texture()?;
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
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            pass.draw(0..VERTICES.len() as u32, 0..1);
        }
        self.queue.submit(Some(encoder.finish()));
        frame.present();
        Ok(())
    }
}

fn main() {
    let event_loop = EventLoop::new().expect("イベントループの作成に失敗しました");
    let window: &'static Window = Box::leak(Box::new(
        event_loop
            .create_window(Window::default_attributes().with_title("wakamore: wgpu 2D demo"))
            .expect("ウィンドウの作成に失敗しました"),
    ));
    let mut renderer = pollster::block_on(Renderer::new(window));

    event_loop
        .run(move |event, event_loop| {
            event_loop.set_control_flow(ControlFlow::Poll);
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => event_loop.exit(),
                    WindowEvent::Resized(size) => renderer.resize(size),
                    WindowEvent::RedrawRequested => match renderer.render() {
                        Ok(()) => {}
                        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                            renderer.resize(window.inner_size());
                        }
                        Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                        Err(wgpu::SurfaceError::Timeout) => {}
                        Err(wgpu::SurfaceError::Other) => {}
                    },
                    _ => {}
                },
                Event::AboutToWait => window.request_redraw(),
                _ => {}
            }
        })
        .expect("イベントループの実行に失敗しました");
}
