//! common: shared input/event types and traits.

use bevy::prelude::*;

pub use bevy::prelude::KeyCode;

pub trait InputEvent: Message + Send + Sync + 'static {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PlayBinding {
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    ScratchUp,
    ScratchDown,
}

#[derive(Event, Message, Clone, Copy, Debug, PartialEq, Eq)]
pub enum TitleInputEvent {
    Confirm,
    Cancel,
}

impl InputEvent for TitleInputEvent {}

#[derive(Event, Message, Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayingInputEvent {
    PlayKeyDown(PlayBinding),
    PlayKeyUp(PlayBinding),
    Abort,
}

impl InputEvent for PlayingInputEvent {}

#[derive(Event, Message, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResultInputEvent {
    Confirm,
    Cancel,
}

impl InputEvent for ResultInputEvent {}

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
