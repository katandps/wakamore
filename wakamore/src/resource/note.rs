use bevy::prelude::*;

use crate::component::LANE_COUNT;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum Lane7S {
    /// Lane 1 (left-most)
    Lane1 = 0,
    /// Lane 2
    Lane2 = 1,
    /// Lane 3
    Lane3 = 2,
    /// Lane 4
    Lane4 = 3,
    /// Lane 5
    Lane5 = 4,
    /// Lane 6
    Lane6 = 5,
    /// Lane 7
    Lane7 = 6,
    /// Special lane (Scratch)
    Scratch = 7,
}

#[derive(Clone, Debug)]
pub struct ChartNote {
    pub lane: Lane7S,
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
                    lane: Lane7S::Lane1,
                    time_from_start_secs: 0.50,
                },
                ChartNote {
                    lane: Lane7S::Lane4,
                    time_from_start_secs: 0.90,
                },
                ChartNote {
                    lane: Lane7S::Lane6,
                    time_from_start_secs: 1.20,
                },
                ChartNote {
                    lane: Lane7S::Scratch,
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
