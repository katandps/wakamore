use bevy::prelude::*;

use crate::component::LANE_COUNT;

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
