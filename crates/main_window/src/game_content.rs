use crate::renderer::{
    Color, Rect,
    image::{ImageDestination, ImageDrawOptions, ImageSource},
    rectangle::RectangleDrawOptions,
};
use std::time::Instant;

pub struct GameContent {
    started_at: Instant,
}

pub enum DrawOptions {
    Image(ImageDrawOptions<'static>),
    Rectangle(RectangleDrawOptions),
}

impl GameContent {
    pub fn new() -> Self {
        Self {
            started_at: Instant::now(),
        }
    }

    pub fn draw_options(&self) -> Vec<DrawOptions> {
        let elapsed = self.started_at.elapsed().as_secs_f32();
        vec![
            DrawOptions::Image(ImageDrawOptions::new(
                DUMMY_IMAGE,
                ImageDestination::new(
                    Rect::new(
                        0.70 * (elapsed * 0.55).sin() - 0.275,
                        0.42 * (elapsed * 0.80).sin() + 0.275,
                        0.55,
                        0.55,
                    ),
                    0.20 + 0.80 * (0.5 + 0.5 * (elapsed * 1.20).sin()),
                ),
            )),
            DrawOptions::Rectangle(RectangleDrawOptions::new(
                Rect::new(0.0, 0.0, 0.5, 0.5),
                Color::new(1.0, 0.0, 0.0, 0.5),
            )),
        ]
    }
}

const DUMMY_IMAGE: ImageSource = ImageSource::new("resources/circles.png", Rect::new(0, 0, 80, 80));
