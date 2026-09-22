use crate::app::RENDER_PERIOD;
use crate::{app::AppState, game_renderer::GameRenderer, loop_manager::LoopManager};
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
    pub fn initialize(event_loop: &ActiveEventLoop) -> Self {
        let game_window: &'static Window = Box::leak(Box::new(
            event_loop
                .create_window(Window::default_attributes().with_title(GAME_WINDOW_TITLE))
                .expect("ゲームウィンドウの作成に失敗しました"),
        ));
        let game_lenderer = pollster::block_on(GameRenderer::new(game_window));

        MainWindow {
            window: game_window,
            renderer: game_lenderer,
            render_loop: LoopManager::new(Instant::now(), RENDER_PERIOD),
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
        if self.render_loop.render_is_pending() {
            self.render_loop.render_set_pending(false);
            if self
                .renderer
                .render(app_state.screen_manager.render_state())
            {
                app_state.performance.record_render();
            }
        }
    }

    pub fn about_to_wait(&mut self, now: Instant) {
        self.render_loop.update_loop_state(now);
        if !self.render_loop.render_is_pending() {
            self.render_loop.render_set_pending(true);
            self.window.request_redraw();
        }
    }
}
