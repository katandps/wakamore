use bevy::prelude::*;

use crate::component::{LANE_COUNT, lane_center_x};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum Lane {
    /// S キー（白）
    S = 0,
    /// D キー（青）
    D = 1,
    /// F キー（白）
    F = 2,
    /// Space キー（青）
    Space = 3,
    /// J キー（白）
    J = 4,
    /// K キー（青）
    K = 5,
    /// L キー（白）
    L = 6,
    /// Shift キー（赤）
    Shift = 7,
}

impl Lane {
    pub fn center_x(self) -> f32 {
        lane_center_x(self as usize)
    }
}

#[derive(Clone, Debug)]
pub struct ChartNote {
    pub lane: Lane,
    pub sfx: String,
    pub time_from_start_secs: f32,
}

#[derive(Resource, Default)]
pub struct NoteChart {
    pub notes: Vec<ChartNote>,
}

impl NoteChart {
    pub fn demo() -> Self {
        Self {
            notes: vec![
                ChartNote {
                    lane: Lane::S,
                    sfx: "kick".to_string(),
                    time_from_start_secs: 0.50,
                },
                ChartNote {
                    lane: Lane::Space,
                    sfx: "snare".to_string(),
                    time_from_start_secs: 0.90,
                },
                ChartNote {
                    lane: Lane::K,
                    sfx: "hihat".to_string(),
                    time_from_start_secs: 1.20,
                },
                ChartNote {
                    lane: Lane::Shift,
                    sfx: "clap".to_string(),
                    time_from_start_secs: 1.70,
                },
            ],
        }
    }
}

#[derive(Resource, Default)]
pub struct ScoreSummary {
    pub pg: u32,
    pub gr: u32,
    pub miss: u32,
    pub score: u32,
}

#[derive(Resource)]
pub struct LaneInputState {
    pub pressed: [bool; LANE_COUNT],
    pub just_pressed: [bool; LANE_COUNT],
}

impl Default for LaneInputState {
    fn default() -> Self {
        Self {
            pressed: [false; LANE_COUNT],
            just_pressed: [false; LANE_COUNT],
        }
    }
}
