pub mod performance;
pub use crate::performance::PerformanceCounter;

#[derive(Default)]
pub struct PlayCore {}

impl PlayCore {
    pub fn receive_event(&mut self, event: PlayEvent) {
        log::info!("Received event: {:?}", event);
        // Handle the received event here
    }
}

#[derive(Debug)]
pub enum PlayEvent {
    None,
    KeyPressed(KeyPressed),
    KeyReleased(KeyReleased),
}

#[derive(Debug)]
pub enum KeyPressed {
    Key1,
}

#[derive(Debug)]
pub enum KeyReleased {
    Key1,
    ToggleDebugWindow,
}

pub struct AppState {
    pub screen_manager: GameMode,
    pub performance: PerformanceCounter,
    pub play_core: PlayCore,
}

pub struct GameMode;

impl GameMode {
    pub fn current_screen_name(&self) -> &str {
        "MainScreen"
    }
}
