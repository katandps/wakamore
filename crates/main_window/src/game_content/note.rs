use super::PlayComponent;
use crate::renderer::{Color, DrawOptions, Rect, rectangle::RectangleDrawOptions};
use std::time::Instant;
pub struct Note {
    pub timing: Instant,
}

impl PlayComponent for Note {
    fn draw_options(&self) -> Vec<DrawOptions> {
        let elapsed = self.timing.elapsed().as_secs_f32();
        vec![DrawOptions::Rectangle(RectangleDrawOptions::new(
            Rect::new(0.0, 0.0 + elapsed / 10.0, 0.3, 0.3),
            Color::new(1.0, 1.0, 0.0, 1.0),
        ))]
    }
}
