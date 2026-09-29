use std::time::Instant;

pub struct Note {
    timing: Instant,
}

impl Note {
    pub fn draw_options(&self) -> Vec<super::DrawOptions> {
        vec![]
    }
}
