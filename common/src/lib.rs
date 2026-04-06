//! common: shared input/event types and traits.

use bevy::prelude::*;

pub use bevy::prelude::KeyCode;

#[derive(Event, Message, Clone, Debug, PartialEq, Eq)]
pub enum InputAction {
    Confirm,
    Cancel,
}

#[derive(Event, Message, Clone, Debug, PartialEq, Eq)]
pub enum InputEvent {
    KeyDown(KeyCode),
    KeyUp(KeyCode),
    Action(InputAction),
}

pub trait InputSink: Send + Sync {
    fn send(&self, ev: InputEvent);
}

#[derive(Event, Message, Clone, Copy)]
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

#[derive(Event, Message, Clone, Copy)]
pub struct LaneJudgementEvent {
    pub lane_index: usize,
    pub kind: JudgementKind,
}
