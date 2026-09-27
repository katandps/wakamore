use crate::renderer::{
    Rect,
    image::{ImageDestination, ImageDrawOptions, ImageSource},
};
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
        ImageDrawOptions::new(
            DUMMY_IMAGE,
            ImageDestination {
                rect: Rect::new(
                    0.70 * (elapsed * 0.55).sin() - 0.275,
                    0.42 * (elapsed * 0.80).sin() + 0.275,
                    0.55,
                    0.55,
                ),
                opacity: 0.20 + 0.80 * (0.5 + 0.5 * (elapsed * 1.20).sin()),
            },
        )
    }
}

const DUMMY_IMAGE: ImageSource = ImageSource {
    path: "resources/circles.png",
    source: Rect::new(0, 0, 80, 80),
};
