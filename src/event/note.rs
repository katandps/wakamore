use bevy::prelude::*;

#[derive(Message, Clone, Copy)]
pub struct LaneInputEvent {
    pub lane_index: usize,
    pub pressed: bool,
}

#[derive(Clone, Copy)]
pub enum JudgementKind {
    Pg,
    Gr,
    Miss,
}

#[derive(Message, Clone, Copy)]
pub struct LaneJudgementEvent {
    pub lane_index: usize,
    pub kind: JudgementKind,
}
