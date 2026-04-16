pub mod fps;
pub mod note;

pub use note::GameplayEntity;

pub use crate::resource::note::ScoreSummary;

pub use crate::system::note_spawn::ChartPlayback;

pub const LANE_COUNT: usize = 8;
pub const WHITE_NOTE_WIDTH: f32 = 60.0;
pub const BLUE_NOTE_WIDTH: f32 = 48.0;
pub const RED_NOTE_WIDTH: f32 = 108.0;
pub const NOTE_HEIGHT: f32 = 24.0;
pub const LANE_GAP: f32 = 3.0;
pub const NOTE_TRAVEL_SECONDS: f32 = 0.5;
pub const RESPAWN_DELAY_MIN_SECONDS: f32 = 0.08;
pub const RESPAWN_DELAY_MAX_SECONDS: f32 = 0.25;
pub const JUDGE_LINE_Y_FROM_BOTTOM: f32 = 200.0;
pub const JUDGE_LINE_THICKNESS: f32 = 4.0;
pub const KEY_INDICATOR_DIAMETER: f32 = 18.0;
pub const KEY_INDICATOR_Y_FROM_BOTTOM: f32 = JUDGE_LINE_Y_FROM_BOTTOM - 30.0;
pub const JUDGEMENT_TEXT_Y_FROM_BOTTOM: f32 = KEY_INDICATOR_Y_FROM_BOTTOM - 24.0;
pub const JUDGEMENT_DISPLAY_SECONDS: f32 = 0.18;
pub const PG_WINDOW_MS: f32 = 20.0;
pub const GR_WINDOW_MS: f32 = 40.0;
pub const SCORE_PG: u32 = 1000;
pub const SCORE_GR: u32 = 500;
pub const LANE_TOTAL_WIDTH: f32 =
    WHITE_NOTE_WIDTH * 4.0 + BLUE_NOTE_WIDTH * 3.0 + RED_NOTE_WIDTH + LANE_GAP * 7.0;

#[derive(Clone, Copy)]
pub struct LaneSpec {
    pub width: f32,
    pub color: bevy::prelude::Color,
}

pub fn lane_specs() -> [LaneSpec; LANE_COUNT] {
    [
        LaneSpec {
            width: WHITE_NOTE_WIDTH,
            color: bevy::prelude::Color::WHITE,
        },
        LaneSpec {
            width: BLUE_NOTE_WIDTH,
            color: bevy::prelude::Color::srgb(0.0, 0.5, 1.0),
        },
        LaneSpec {
            width: WHITE_NOTE_WIDTH,
            color: bevy::prelude::Color::WHITE,
        },
        LaneSpec {
            width: BLUE_NOTE_WIDTH,
            color: bevy::prelude::Color::srgb(0.0, 0.5, 1.0),
        },
        LaneSpec {
            width: WHITE_NOTE_WIDTH,
            color: bevy::prelude::Color::WHITE,
        },
        LaneSpec {
            width: BLUE_NOTE_WIDTH,
            color: bevy::prelude::Color::srgb(0.0, 0.5, 1.0),
        },
        LaneSpec {
            width: WHITE_NOTE_WIDTH,
            color: bevy::prelude::Color::WHITE,
        },
        LaneSpec {
            width: RED_NOTE_WIDTH,
            color: bevy::prelude::Color::srgb(1.0, 0.0, 0.0),
        },
    ]
}

pub fn lane_center_x(lane_index: usize) -> f32 {
    let mut left = -LANE_TOTAL_WIDTH * 0.5;

    for (idx, lane) in lane_specs().into_iter().enumerate() {
        let center = left + lane.width * 0.5;
        if idx == lane_index {
            return center;
        }
        left += lane.width + LANE_GAP;
    }

    0.0
}
