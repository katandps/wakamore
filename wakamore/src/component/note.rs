use bevy::prelude::*;

#[derive(Component)]
pub struct Note {
    pub(crate) lane_index: usize,
    // The scheduled time (in seconds from playback start) when this note should be judged.
    // Used to compute exact drawing position from playback time and speed.
    pub(crate) scheduled_time_secs: f32,
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
