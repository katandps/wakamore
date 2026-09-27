use crate::game_renderer::{ImageDrawOptions, Rect};
use std::time::Instant;

pub struct GameContent {
    started_at: Instant,
}

impl GameContent {
    pub fn new() -> Self {
        Self {
            started_at: Instant::now(),
        }
    }

    pub fn draw_options(&self) -> ImageDrawOptions<'static> {
        let elapsed = self.started_at.elapsed().as_secs_f32();
        ImageDrawOptions {
            path: "resources/circles.png",
            source: Rect {
                position: [0, 0],
                size: [80, 80],
            },
            destination: Rect {
                position: [
                    0.70 * (elapsed * 0.55).sin() - 0.275,
                    0.42 * (elapsed * 0.80).sin() + 0.275,
                ],
                size: [0.55, 0.55],
            },
            opacity: 0.20 + 0.80 * (0.5 + 0.5 * (elapsed * 1.20).sin()),
        }
    }
}
