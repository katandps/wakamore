//! common: shared input/event types and traits.

use bevy::prelude::*;

pub use bevy::prelude::KeyCode;

pub trait InputEvent: Message + Send + Sync + 'static {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ScreenId {
    Title,
    Playing,
    Result,
}

#[derive(Clone, Debug)]
pub enum RawInput {
    Key(KeyCode),
    Gamepad(bevy::input::gamepad::GamepadButton),
}

#[derive(Event, Message, Clone, Debug)]
pub struct LaneInputEvent {
    pub lane_index: usize,
    pub pressed: bool,
    pub raw: Option<RawInput>,
}

#[derive(Clone, Copy, Debug)]
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

#[derive(Clone, Debug)]
pub struct LogEntry {
    pub timestamp: f64,
    pub lane_index: usize,
    pub raw: Option<RawInput>,
    pub judgement: Option<JudgementKind>,
}

use std::collections::HashMap;

#[derive(Resource, Default, Debug)]
pub struct InputLog(pub Vec<LogEntry>);

#[derive(Resource, Default, Debug)]
pub struct LastRawByLane(pub HashMap<usize, RawInput>);
