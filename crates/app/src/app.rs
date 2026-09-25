use crate::debug_window::DebugWindow;
use crate::main_window::MainWindow;
use crate::screen::ScreenCommand;
use crate::screen::ScreenManager;
use debug::PerformanceCounter;
use loop_manager::LoopManager;
use play_core::{KeyPressed, KeyReleased, PlayCore, PlayEvent};
use std::time::Duration;
use std::time::Instant;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::WindowId,
};

pub const MAIN_LOOP_PERIOD: Duration = Duration::from_nanos(1_000_000_000 / 3000);
pub const RENDER_PERIOD: Duration = Duration::from_nanos(1_000_000_000 / 240);

pub struct App {
    main_window: Option<MainWindow>,
    debug_window: Option<DebugWindow>,
    app_state: AppState,
    main_loop: LoopManager,
}

pub struct AppState {
    pub screen_manager: ScreenManager,
    pub performance: PerformanceCounter,
    play_core: PlayCore,
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

        self.main_window = Some(MainWindow::initialize(
            event_loop,
            LoopManager::new(Instant::now(), RENDER_PERIOD),
        ));
        self.debug_window = Some(DebugWindow::initialize(
            event_loop,
            LoopManager::new(Instant::now(), RENDER_PERIOD),
        ));
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
        if let Some(main_window) = self.main_window.as_mut() {
            if main_window.window.id() == window_id {
                main_window.handle_event(event_loop, &event, &mut self.app_state);
            }
        }
        if let Some(debug_window) = self.debug_window.as_mut() {
            if debug_window.window.id() == window_id {
                debug_window.handle_event(
                    event_loop,
                    &event,
                    &self.app_state,
                    self.main_window
                        .as_ref()
                        .map_or_default(|w| w.window.outer_size()),
                );
            }
        }
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
            _ => match self.app_state.screen_manager.handle_event(&event) {
                ScreenCommand::None | ScreenCommand::Replace(_) => {}
            },
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

        event_loop.set_control_flow(ControlFlow::WaitUntil(self.main_loop.next_loop_at));
    }
}

pub fn run() {
    env_logger::init();
    let event_loop = EventLoop::<PlayEvent>::with_user_event()
        .build()
        .expect("イベントループの作成に失敗しました");
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop
        .run_app(&mut App::new())
        .expect("イベントループの実行に失敗しました");
}
