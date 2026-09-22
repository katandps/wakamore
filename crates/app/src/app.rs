use crate::screen::ScreenManager;
use crate::{game_renderer::GameRenderer, screen::ScreenCommand};
use debug::{DebugRenderer, PerformanceCounter};
use play_core::{KeyPressed, KeyReleased, PlayCore, PlayEvent};
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

use crate::loop_manager::LoopManager;
use std::time::Duration;

pub const MAIN_LOOP_PERIOD: Duration = Duration::from_micros(50);
pub const RENDER_PERIOD: Duration = Duration::from_nanos(8_333_333);

pub struct App {
    main_window: Option<MainWindow>,
    debug_window: Option<DebugWindow>,
    app_state: AppState,
    main_loop: LoopManager,
}

struct AppState {
    screen_manager: ScreenManager,
    performance: PerformanceCounter,
    play_core: PlayCore,
}

struct MainWindow {
    window: &'static Window,
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
            if app_state.screen_manager.render(&mut self.renderer) {
                app_state.performance.record_render();
            }
        }
    }

    fn about_to_wait(&mut self, now: Instant) {
        self.render_loop.update_loop_state(now);
        if !self.render_loop.render_is_pending() {
            self.render_loop.render_set_pending(true);
            self.window.request_redraw();
        }
    }
}

struct DebugWindow {
    window: &'static Window,
    renderer: DebugRenderer,
    render_loop: LoopManager,
    visible: bool,
}

impl DebugWindow {
    pub fn initialize(event_loop: &ActiveEventLoop) -> Self {
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
            render_loop: LoopManager::new(Instant::now(), RENDER_PERIOD),
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

    fn handle_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        event: &WindowEvent,
        app_state: &AppState,
    ) {
        self.renderer.handle_window_event(self.window, event);
        match event {
            WindowEvent::CloseRequested => self.handle_window_close(),
            WindowEvent::Resized(size) => self.renderer.resize(size.clone()),
            WindowEvent::RedrawRequested => self.handle_window_redraw(app_state),
            _ => {}
        }

        let _ = event_loop;
    }

    fn handle_window_close(&mut self) {
        self.visible = false;
        self.render_loop.render_set_pending(false);
        self.window.set_visible(false);
    }

    fn handle_window_redraw(&mut self, app_state: &AppState) {
        if self.render_loop.render_is_pending() {
            self.render_loop.render_set_pending(false);
            let main_window_size = self.window.inner_size();
            let _ = self.renderer.render(
                self.window,
                &app_state.performance,
                app_state.screen_manager.current_screen_name(),
                main_window_size,
            );
        }
    }

    fn about_to_wait(&mut self, now: Instant) {
        if !self.visible {
            return;
        }
        self.render_loop.update_loop_state(now);
        if !self.render_loop.render_is_pending() {
            self.render_loop.render_set_pending(true);
            self.window.request_redraw();
        }
    }
}

impl App {
    fn new() -> Self {
        Self {
            main_window: None,
            debug_window: None,
            app_state: AppState {
                screen_manager: ScreenManager::new(),
                performance: PerformanceCounter::default(),
                play_core: PlayCore::default(),
            },
            main_loop: LoopManager::new(Instant::now(), MAIN_LOOP_PERIOD),
        }
    }

    fn create_windows(&mut self, event_loop: &ActiveEventLoop) {
        if self.main_window.is_some() && self.debug_window.is_some() {
            return;
        }

        self.main_window = Some(MainWindow::initialize(event_loop));
        self.debug_window = Some(DebugWindow::initialize(event_loop));
    }
}

impl ApplicationHandler<PlayEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.create_windows(event_loop);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            // キーボード入力
            WindowEvent::KeyboardInput { event, .. } => {
                if !event.repeat {
                    if event.state == ElementState::Pressed {
                        let key_event = match event.physical_key {
                            PhysicalKey::Code(KeyCode::F12) => PlayEvent::None,
                            _ => PlayEvent::KeyPressed(KeyPressed::Key1),
                        };
                        self.user_event(event_loop, key_event);
                    } else if event.state == ElementState::Released {
                        let key_event = match event.physical_key {
                            PhysicalKey::Code(KeyCode::F12) => {
                                PlayEvent::KeyReleased(KeyReleased::ToggleDebugWindow)
                            }
                            _ => PlayEvent::KeyReleased(KeyReleased::Key1),
                        };
                        self.user_event(event_loop, key_event);
                    }
                }
            }
            // マウス入力(まだ実装しない)
            // ゲームパッド入力(まだ実装しない)
            _ => {
                if let Some(main_window) = self.main_window.as_mut() {
                    if main_window.window.id() == window_id {
                        main_window.handle_event(event_loop, &event, &mut self.app_state);
                    }
                }
                if let Some(debug_window) = self.debug_window.as_mut() {
                    if debug_window.window.id() == window_id {
                        debug_window.handle_event(event_loop, &event, &self.app_state);
                    }
                }
                match self.app_state.screen_manager.handle_event(&event) {
                    ScreenCommand::None | ScreenCommand::Replace(_) => {}
                }
            }
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: PlayEvent) {
        match event {
            PlayEvent::KeyReleased(KeyReleased::ToggleDebugWindow) => self
                .debug_window
                .iter_mut()
                .for_each(|window| window.toggle_visibility()),
            _ => self.app_state.play_core.receive_event(event),
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let now = Instant::now();
        if self.main_loop.next_loop_is_came(now) {
            self.app_state.performance.record_loop();
            self.app_state.screen_manager.update();
            self.main_loop.update_loop_state(now);
        }
        self.main_window
            .iter_mut()
            .for_each(|window| window.about_to_wait(now));
        self.debug_window
            .iter_mut()
            .for_each(|window| window.about_to_wait(now));

        event_loop.set_control_flow(ControlFlow::WaitUntil(self.main_loop.next_wait_deadline()));
    }
}

pub fn run() {
    env_logger::init();
    let event_loop = EventLoop::<PlayEvent>::with_user_event()
        .build()
        .expect("イベントループの作成に失敗しました");
    event_loop.set_control_flow(ControlFlow::Wait);
    event_loop
        .run_app(&mut App::new())
        .expect("イベントループの実行に失敗しました");
}
