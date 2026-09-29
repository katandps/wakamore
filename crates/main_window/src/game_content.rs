mod judge_line;
mod note;

use crate::renderer::{
    Color, DrawOptions, Rect,
    image::{ImageDestination, ImageDrawOptions, ImageSource},
    rectangle::RectangleDrawOptions,
};
use std::time::Instant;

pub struct GameContent {
    started_at: Instant,
    judge_line: judge_line::JudgeLine,
    notes: Vec<note::Note>,
}

impl GameContent {
    pub fn new() -> Self {
        Self {
            started_at: Instant::now(),
            judge_line: judge_line::JudgeLine,
            notes: vec![note::Note {
                timing: Instant::now(),
            }],
        }
    }

    pub fn draw_options(&self) -> Vec<DrawOptions> {
        let elapsed = self.started_at.elapsed().as_secs_f32();
        let judge_line = self.judge_line.draw_options();
        let notes = self.notes.iter().flat_map(|note| note.draw_options());
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
        .into_iter()
        .chain(judge_line)
        .chain(notes)
        .collect()
    }
}

const DUMMY_IMAGE: ImageSource = ImageSource::new("resources/circles.png", Rect::new(0, 0, 80, 80));

trait PlayComponent {
    fn draw_options(&self) -> Vec<DrawOptions>;
}
