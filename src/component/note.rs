use bevy::prelude::*;

#[derive(Component)]
pub struct Note {
    pub(crate) lane_index: usize,
    pub(crate) initialized: bool,
    pub(crate) respawn_delay_remaining: f32,
}

#[derive(Component)]
pub struct GameplayEntity;

#[derive(Component)]
pub struct LaneKeyIndicator {
    pub(crate) index: usize,
}

#[derive(Component)]
pub struct LaneJudgeText {
    pub(crate) index: usize,
    pub(crate) remaining_secs: f32,
}

#[derive(Component)]
pub struct JudgeLine;
