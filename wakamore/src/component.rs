pub mod fps;
pub mod note;

pub use wakamore_render::GameplayEntity;
pub use wakamore_render::{
    JUDGE_LINE_Y_FROM_BOTTOM, LANE_COUNT, NOTE_HEIGHT, NOTE_TRAVEL_SECONDS, lane_center_x,
    lane_specs,
};

pub use crate::resource::note::ScoreSummary;

pub use crate::system::note_spawn::ChartPlayback;
pub const PG_WINDOW_MS: f32 = 20.0;
pub const GR_WINDOW_MS: f32 = 40.0;
pub const SCORE_PG: u32 = 1000;
pub const SCORE_GR: u32 = 500;
