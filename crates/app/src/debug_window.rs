use crate::app::AppState;
use debug::DebugRenderer;
use loop_manager::LoopManager;
use std::time::Instant;
use winit::{dpi::PhysicalSize, event::WindowEvent, event_loop::ActiveEventLoop, window::Window};
const DEBUG_WINDOW_TITLE: &str = "wakamore: debug";

pub struct DebugWindow {
    pub window: &'static Window,
    renderer: DebugRenderer,
    render_loop: LoopManager,
    visible: bool,
}

impl DebugWindow {
    pub fn initialize(event_loop: &ActiveEventLoop, render_loop: LoopManager) -> Self {
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

        let debug_renderer = pollster::block_on(DebugRenderer::new(debug_window));
        DebugWindow {
            window: debug_window,
            renderer: debug_renderer,
            render_loop: render_loop,
            visible: true,
        }
    }

    pub fn toggle_visibility(&mut self) {
        self.visible = !self.visible;
        self.window.set_visible(self.visible);
        if self.visible {
            self.window.request_redraw();
        }
    }

    pub fn handle_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        event: &WindowEvent,
        app_state: &AppState,
        main_window_size: PhysicalSize<u32>,
    ) {
        self.renderer.handle_window_event(self.window, event);
        match event {
            WindowEvent::CloseRequested => self.handle_window_close(),
            WindowEvent::Resized(size) => self.renderer.resize(size.clone()),
            WindowEvent::RedrawRequested => self.handle_window_redraw(app_state, main_window_size),
            _ => {}
        }

        let _ = event_loop;
    }

    fn handle_window_close(&mut self) {
        self.visible = false;
        self.window.set_visible(false);
    }

    fn handle_window_redraw(&mut self, app_state: &AppState, main_window_size: PhysicalSize<u32>) {
        let now = Instant::now();
        if self.render_loop.next_loop_is_came(now) {
            let _ = self.renderer.render(
                self.window,
                &app_state.performance,
                app_state.screen_manager.current_screen_name(),
                main_window_size,
            );
            self.render_loop.update_loop_state(now);
        }
    }

    pub fn about_to_wait(&mut self, now: Instant) {
        if !self.visible {
            return;
        }
        if self.render_loop.next_loop_is_came(now) {
            self.window.request_redraw();
        }
    }
}
