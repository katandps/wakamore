use crate::{app::AppState, game_renderer::GameRenderer};
use loop_manager::LoopManager;
use std::time::Instant;
use winit::window::Window;
use winit::{event::WindowEvent, event_loop::ActiveEventLoop};

const GAME_WINDOW_TITLE: &str = "wakamore: game";

pub struct MainWindow {
    pub window: &'static Window,
    renderer: GameRenderer,
    render_loop: LoopManager,
}

impl MainWindow {
    pub fn initialize(event_loop: &ActiveEventLoop, render_loop: LoopManager) -> Self {
        let game_window: &'static Window = Box::leak(Box::new(
            event_loop
                .create_window(Window::default_attributes().with_title(GAME_WINDOW_TITLE))
                .expect("ゲームウィンドウの作成に失敗しました"),
        ));
        let game_lenderer = pollster::block_on(GameRenderer::new(game_window));

        MainWindow {
            window: game_window,
            renderer: game_lenderer,
            render_loop: render_loop,
        }
    }

    pub fn handle_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        event: &WindowEvent,
        app_state: &mut AppState,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => self.renderer.resize(size.clone()),
            WindowEvent::RedrawRequested => self.handle_redraw_requested(app_state),
            _ => (),
        }
    }

    fn handle_redraw_requested(&mut self, app_state: &mut AppState) {
        let now = Instant::now();
        if self.render_loop.next_loop_is_came(now) {
            if self
                .renderer
                .render(app_state.screen_manager.render_state())
            {
                app_state.performance.record_render();
            }
            self.render_loop.update_loop_state(now);
        }
    }

    pub fn about_to_wait(&mut self, now: Instant) {
        if self.render_loop.next_loop_is_came(now) {
            self.window.request_redraw();
        }
    }
}
