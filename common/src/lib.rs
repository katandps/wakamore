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

/// Trait to map a physical `KeyCode` to a high-level `InputAction`.
/// Implementations live in the `input` crate and are registered as resources.
pub trait KeyToAction: Send + Sync {
    fn key_to_action(&self, key: KeyCode) -> Option<InputAction>;
    /// Returns the set of physical keys that this mapping will report.
    /// Used by `input` to know which keys to poll in addition to the
    /// hardcoded tracked keys.
    fn bound_keys(&self) -> Vec<KeyCode>;
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

#[derive(Resource)]
pub struct KeyToActionResource(pub Box<dyn KeyToAction>);
