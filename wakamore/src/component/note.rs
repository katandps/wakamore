use bevy::prelude::*;

#[derive(Component)]
pub struct Note {
    pub(crate) lane_index: usize,
    // The scheduled time (in seconds from playback start) when this note should be judged.
    // Used to compute exact drawing position from playback time and speed.
    pub(crate) scheduled_time_secs: f32,
}
