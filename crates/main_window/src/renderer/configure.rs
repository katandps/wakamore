use crate::renderer::rectangle::RectangleVertex;

pub async fn request_adapter(
    instance: &wgpu::Instance,
    surface: &wgpu::Surface<'_>,
) -> wgpu::Adapter {
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

pub async fn request_device(adapter: &wgpu::Adapter) -> (wgpu::Device, wgpu::Queue) {
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

pub fn configure_surface(
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

pub fn image_pipeline(
    device: &wgpu::Device,
    shader: &wgpu::ShaderModule,
    config: &wgpu::SurfaceConfiguration,
    texture_bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let image_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
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
            buffers: &[Some(crate::renderer::ImageVertex::layout())],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
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
    image_pipeline
}

pub fn rectangle_image_pipeline(
    device: &wgpu::Device,
    shader: &wgpu::ShaderModule,
    config: &wgpu::SurfaceConfiguration,
) -> wgpu::RenderPipeline {
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
                format: config.format,
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
    rectangle_pipeline
}
