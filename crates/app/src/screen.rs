use crate::game_renderer::GameRenderState;
use winit::event::{ElementState, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

pub trait GameScreen {
    fn update(&mut self) -> ScreenCommand {
        ScreenCommand::None
    }

    fn handle_event(&mut self, event: &WindowEvent) -> ScreenCommand;

    fn render_state(&self) -> GameRenderState;

    fn name(&self) -> &'static str;
}

pub enum ScreenCommand {
    None,
    Replace(Box<dyn GameScreen>),
}

pub struct ScreenManager {
    current: Box<dyn GameScreen>,
}

impl ScreenManager {
    pub fn new() -> Self {
        Self {
            current: Box::new(TitleScreen),
        }
    }

    pub fn handle_event(&mut self, event: &WindowEvent) -> ScreenCommand {
        let command = self.current.handle_event(event);
        self.apply(command)
    }

    pub fn update(&mut self) {
        let command = self.current.update();
        self.apply(command);
    }

    pub fn render_state(&self) -> GameRenderState {
        self.current.render_state()
    }

    pub fn current_screen_name(&self) -> &'static str {
        self.current.name()
    }

    fn apply(&mut self, command: ScreenCommand) -> ScreenCommand {
        match command {
            ScreenCommand::Replace(screen) => {
                self.current = screen;
                ScreenCommand::None
            }
            other => other,
        }
    }
}

struct TitleScreen;

impl GameScreen for TitleScreen {
    fn handle_event(&mut self, event: &WindowEvent) -> ScreenCommand {
        if is_pressed(event, KeyCode::Enter) {
            ScreenCommand::Replace(Box::new(GameplayScreen))
        } else {
            ScreenCommand::None
        }
    }

    fn render_state(&self) -> GameRenderState {
        GameRenderState::Title
    }

    fn name(&self) -> &'static str {
        "Title"
    }
}

struct GameplayScreen;

impl GameScreen for GameplayScreen {
    fn handle_event(&mut self, event: &WindowEvent) -> ScreenCommand {
        if is_pressed(event, KeyCode::Escape) {
            ScreenCommand::Replace(Box::new(TitleScreen))
        } else {
            ScreenCommand::None
        }
    }

    fn render_state(&self) -> GameRenderState {
        GameRenderState::Gameplay
    }

    fn name(&self) -> &'static str {
        "Gameplay"
    }
}

fn is_pressed(event: &WindowEvent, key: KeyCode) -> bool {
    matches!(
        event,
        WindowEvent::KeyboardInput { event, .. }
            if event.state == ElementState::Pressed
                && event.physical_key == PhysicalKey::Code(key)
    )
}
