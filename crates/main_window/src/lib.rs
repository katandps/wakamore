mod game_content;
mod game_renderer;
mod renderer;

use crate::game_content::GameContent;
use crate::game_renderer::GameRenderer;
use loop_manager::LoopManager;
use play_core::AppState;
use std::time::Instant;
use winit::window::Window;
use winit::{event::WindowEvent, event_loop::ActiveEventLoop};

const GAME_WINDOW_TITLE: &str = "wakamore: game";

pub struct MainWindow {
    pub window: &'static Window,
    renderer: GameRenderer,
    content: GameContent,
    render_loop: LoopManager,
}

impl MainWindow {
    pub async fn initialize(event_loop: &ActiveEventLoop, render_loop: LoopManager) -> Self {
        let game_window: &'static Window = Box::leak(Box::new(
            event_loop
                .create_window(Window::default_attributes().with_title(GAME_WINDOW_TITLE))
                .expect("ゲームウィンドウの作成に失敗しました"),
        ));
        let game_renderer = GameRenderer::new(game_window).await;

        MainWindow {
            window: game_window,
            renderer: game_renderer,
            content: GameContent::new(),
            render_loop,
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
            WindowEvent::Resized(size) => self.renderer.resize(*size),
            WindowEvent::RedrawRequested => self.handle_redraw_requested(app_state),
            _ => (),
        }
    }

    fn handle_redraw_requested(&mut self, app_state: &mut AppState) {
        let now = Instant::now();
        if self.render_loop.next_loop_is_came(now) {
            let options = self.content.draw_options();
            let rendered = if let Some(mut frame) = self.renderer.begin_frame() {
                frame.draw_image(&options);
                frame.end_frame()
            } else {
                false
            };
            if rendered {
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
