use crate::performance::PerformanceCounter;
use egui_wgpu::{Renderer, RendererOptions, ScreenDescriptor};
use egui_winit::State as EguiState;
use winit::window::Window;

pub struct DebugRenderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    egui_renderer: Renderer,
}

impl DebugRenderer {
    pub async fn new(window: &'static Window) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let surface = instance
            .create_surface(window)
            .expect("デバッグ Surface の作成に失敗しました");
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                apply_limit_buckets: false,
            })
            .await
            .expect("デバッグウィンドウ用 GPU アダプターがありません");
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("debug wgpu device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::Performance,
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                trace: wgpu::Trace::Off,
            })
            .await
            .expect("デバッグウィンドウ用 GPU デバイスの作成に失敗しました");
        let capabilities = surface.get_capabilities(&adapter);
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
        surface.configure(&device, &config);
        let egui_renderer = Renderer::new(&device, config.format, RendererOptions::default());

        Self {
            surface,
            device,
            queue,
            config,
            egui_renderer,
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

    pub fn render(
        &mut self,
        window: &Window,
        context: &egui::Context,
        egui_state: &mut EguiState,
        performance: &PerformanceCounter,
        current_screen: &str,
        main_window_size: winit::dpi::PhysicalSize<u32>,
    ) -> bool {
        let raw_input = egui_state.take_egui_input(window);
        let mut full_output = context.run_ui(raw_input, |ui| {
            egui::CentralPanel::default().show(ui, |ui| {
                ui.heading("wakamore debug");
                ui.separator();
                egui::Grid::new("debug_metrics")
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("Main loop");
                        ui.label(format!("{:.1} Hz", performance.loops_per_second));
                        ui.end_row();
                        ui.label("Render");
                        ui.label(format!("{:.1} FPS", performance.frames_per_second));
                        ui.end_row();
                        ui.label("Window size");
                        ui.label(format!(
                            "{} x {}",
                            main_window_size.width, main_window_size.height
                        ));
                        ui.end_row();
                        ui.label("Current screen");
                        ui.label(current_screen);
                        ui.end_row();
                    });
                ui.add_space(12.0);
                ui.label("F12: toggle this debug window");
            });
        });
        egui_state.handle_platform_output(window, full_output.platform_output);
        let paint_jobs = context.tessellate(full_output.shapes, full_output.pixels_per_point);

        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame)
            | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(&self.device, &self.config);
                full_output.textures_delta.clear();
                return false;
            }
            wgpu::CurrentSurfaceTexture::Timeout
            | wgpu::CurrentSurfaceTexture::Occluded
            | wgpu::CurrentSurfaceTexture::Validation => {
                full_output.textures_delta.clear();
                return false;
            }
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let descriptor = ScreenDescriptor {
            size_in_pixels: [self.config.width, self.config.height],
            pixels_per_point: full_output.pixels_per_point,
        };
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("debug egui encoder"),
            });
        for (id, deltas) in &full_output.textures_delta.set {
            for delta in deltas {
                self.egui_renderer
                    .update_texture(&self.device, &self.queue, *id, delta);
            }
        }
        let user_cmd_bufs = self.egui_renderer.update_buffers(
            &self.device,
            &self.queue,
            &mut encoder,
            &paint_jobs,
            &descriptor,
        );
        {
            let pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("debug egui pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.03,
                            g: 0.04,
                            b: 0.06,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                multiview_mask: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            self.egui_renderer
                .render(&mut pass.forget_lifetime(), &paint_jobs, &descriptor);
        }
        let mut command_buffers = vec![encoder.finish()];
        command_buffers.extend(user_cmd_bufs);
        self.queue.submit(command_buffers);
        for id in &full_output.textures_delta.free {
            self.egui_renderer.free_texture(id);
        }
        full_output.textures_delta.clear();
        self.queue.present(frame);
        true
    }
}
