use crate::renderer::{Color, DrawOptions, Rect, rectangle::RectangleDrawOptions};

pub struct JudgeLine;
impl JudgeLine {
    pub fn draw_options(&self) -> Vec<DrawOptions> {
        vec![DrawOptions::Rectangle(RectangleDrawOptions::new(
            Rect::new(0.0, 0.0, 1.0, 0.1),
            Color::new(1.0, 1.0, 1.0, 1.0),
        ))]
    }
}
