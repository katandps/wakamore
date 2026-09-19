use crate::debug_renderer::DebugRenderer;
use crate::game_renderer::GameRenderer;
use crate::performance::{MAIN_LOOP_PERIOD, PerformanceCounter, RENDER_PERIOD};
use crate::screen::{ScreenCommand, ScreenManager};
use std::time::Instant;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

const GAME_WINDOW_TITLE: &str = "wakamore: game";
const DEBUG_WINDOW_TITLE: &str = "wakamore: debug";

/// メインループ・描画の起動タイミングと再描画要求の保留状態をまとめる。
struct FrameTiming {
    next_loop_at: Instant,
    next_render_at: Instant,
    game_render_pending: bool,
    debug_render_pending: bool,
}

impl FrameTiming {
    fn new(now: Instant) -> Self {
        Self {
            next_loop_at: now,
            next_render_at: now,
            game_render_pending: false,
            debug_render_pending: false,
        }
    }

    fn next_wait_deadline(&self) -> Instant {
        self.next_loop_at.min(self.next_render_at)
    }
}

pub struct App {
    game_window: Option<&'static Window>,
    debug_window: Option<&'static Window>,
    game_renderer: Option<GameRenderer>,
    screen_manager: ScreenManager,
    debug_renderer: Option<DebugRenderer>,
    egui_context: egui::Context,
    egui_state: Option<egui_winit::State>,
    performance: PerformanceCounter,
    frame_timing: FrameTiming,
    debug_visible: bool,
}

impl App {
    fn new() -> Self {
        let now = Instant::now();
        Self {
            game_window: None,
            debug_window: None,
            game_renderer: None,
            screen_manager: ScreenManager::new(),
            debug_renderer: None,
            egui_context: egui::Context::default(),
            egui_state: None,
            performance: PerformanceCounter::new(),
            frame_timing: FrameTiming::new(now),
            debug_visible: true,
        }
    }

    fn toggle_debug_window(&mut self) {
        self.debug_visible = !self.debug_visible;
        if let Some(window) = self.debug_window {
            window.set_visible(self.debug_visible);
            if self.debug_visible {
                window.request_redraw();
            }
        }
    }

    fn create_windows(&mut self, event_loop: &ActiveEventLoop) {
        if self.game_window.is_some() {
            return;
        }

        let game_window: &'static Window = Box::leak(Box::new(
            event_loop
                .create_window(Window::default_attributes().with_title(GAME_WINDOW_TITLE))
                .expect("ゲームウィンドウの作成に失敗しました"),
        ));
        let debug_window: &'static Window = Box::leak(Box::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title(DEBUG_WINDOW_TITLE)
                        .with_inner_size(winit::dpi::PhysicalSize::new(360, 240))
                        .with_active(false),
                )
                .expect("デバッグウィンドウの作成に失敗しました"),
        ));

        self.game_renderer = Some(pollster::block_on(GameRenderer::new(game_window)));
        self.debug_renderer = Some(pollster::block_on(DebugRenderer::new(debug_window)));
        self.egui_state = Some(egui_winit::State::new(
            self.egui_context.clone(),
            egui::ViewportId::ROOT,
            debug_window,
            Some(debug_window.scale_factor() as f32),
            debug_window.theme(),
            None,
        ));
        self.game_window = Some(game_window);
        self.debug_window = Some(debug_window);
    }

    fn handle_game_event(&mut self, event_loop: &ActiveEventLoop, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(renderer) = self.game_renderer.as_mut() {
                    renderer.resize(size);
                }
            }
            WindowEvent::KeyboardInput { event, .. }
                if event.state == ElementState::Pressed
                    && event.physical_key == PhysicalKey::Code(KeyCode::F12) =>
            {
                self.toggle_debug_window();
            }
            WindowEvent::RedrawRequested if self.frame_timing.game_render_pending => {
                self.frame_timing.game_render_pending = false;
                if let Some(renderer) = self.game_renderer.as_mut()
                    && self.screen_manager.render(renderer)
                {
                    self.performance.record_render();
                }
            }
            event => match self.screen_manager.handle_event(&event) {
                ScreenCommand::None | ScreenCommand::Replace(_) => {}
            },
        }
    }

    fn handle_debug_event(&mut self, event_loop: &ActiveEventLoop, event: WindowEvent) {
        if let (Some(window), Some(state)) = (self.debug_window, self.egui_state.as_mut()) {
            let _ = state.on_window_event(window, &event);
        }

        match event {
            WindowEvent::CloseRequested => {
                self.debug_visible = false;
                self.frame_timing.debug_render_pending = false;
                if let Some(window) = self.debug_window {
                    window.set_visible(false);
                }
            }
            WindowEvent::Resized(size) => {
                if let Some(renderer) = self.debug_renderer.as_mut() {
                    renderer.resize(size);
                }
            }
            WindowEvent::RedrawRequested if self.frame_timing.debug_render_pending => {
                self.frame_timing.debug_render_pending = false;
                if let (Some(renderer), Some(window), Some(state)) = (
                    self.debug_renderer.as_mut(),
                    self.debug_window,
                    self.egui_state.as_mut(),
                ) {
                    let main_window_size =
                        self.game_window.map(Window::inner_size).unwrap_or_default();
                    let _ = renderer.render(
                        window,
                        &self.egui_context,
                        state,
                        &self.performance,
                        self.screen_manager.current_screen_name(),
                        main_window_size,
                    );
                }
            }
            _ => {}
        }

        let _ = event_loop;
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.create_windows(event_loop);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if self.game_window.map(Window::id) == Some(window_id) {
            self.handle_game_event(event_loop, event);
        } else if self.debug_window.map(Window::id) == Some(window_id) {
            self.handle_debug_event(event_loop, event);
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let now = Instant::now();
        if now >= self.frame_timing.next_loop_at {
            self.performance.record_loop();
            self.screen_manager.update();
            self.frame_timing.next_loop_at =
                next_deadline(now, self.frame_timing.next_loop_at, MAIN_LOOP_PERIOD);
        }

        if now >= self.frame_timing.next_render_at {
            self.frame_timing.next_render_at =
                next_deadline(now, self.frame_timing.next_render_at, RENDER_PERIOD);
            if !self.frame_timing.game_render_pending {
                self.frame_timing.game_render_pending = true;
                if let Some(window) = self.game_window {
                    window.request_redraw();
                }
            }
            if self.debug_visible && !self.frame_timing.debug_render_pending {
                self.frame_timing.debug_render_pending = true;
                if let Some(window) = self.debug_window {
                    window.request_redraw();
                }
            }
        }

        event_loop.set_control_flow(ControlFlow::WaitUntil(
            self.frame_timing.next_wait_deadline(),
        ));
    }
}

fn next_deadline(now: Instant, previous: Instant, period: std::time::Duration) -> Instant {
    let next = previous + period;
    if next <= now { now + period } else { next }
}

pub fn run() {
    let event_loop = EventLoop::new().expect("イベントループの作成に失敗しました");
    event_loop.set_control_flow(ControlFlow::Wait);
    event_loop
        .run_app(&mut App::new())
        .expect("イベントループの実行に失敗しました");
}
