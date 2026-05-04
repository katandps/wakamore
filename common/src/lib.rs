//! common: shared input/event types and traits.

use bevy::prelude::*;

pub use bevy::prelude::KeyCode;

#[derive(Event, Message, Clone, Debug, PartialEq, Eq)]
pub enum InputAction {
    Confirm,
    Cancel,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PlayKey {
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ScratchKey {
    Scratch,
}

impl PlayKey {
    pub fn lane_index(self) -> usize {
        match self {
            PlayKey::Key1 => 0,
            PlayKey::Key2 => 1,
            PlayKey::Key3 => 2,
            PlayKey::Key4 => 4,
            PlayKey::Key5 => 5,
            PlayKey::Key6 => 6,
            PlayKey::Key7 => 7,
        }
    }

    pub fn from_keycode(key: KeyCode) -> Option<Self> {
        use KeyCode::*;
        Some(match key {
            KeyS => PlayKey::Key1,
            KeyD => PlayKey::Key2,
            KeyF => PlayKey::Key3,
            KeyJ => PlayKey::Key4,
            KeyK => PlayKey::Key5,
            KeyL => PlayKey::Key6,
            ShiftLeft | ShiftRight => PlayKey::Key7,
            _ => return None,
        })
    }
}

impl ScratchKey {
    pub fn lane_index(self) -> usize {
        3
    }

    pub fn from_keycode(key: KeyCode) -> Option<Self> {
        use KeyCode::*;
        match key {
            Space => Some(ScratchKey::Scratch),
            _ => None,
        }
    }
}

#[derive(Event, Message, Clone, Debug, PartialEq, Eq)]
pub enum InputEvent {
    PlayKeyDown(PlayKey),
    PlayKeyUp(PlayKey),
    ScratchDown(ScratchKey),
    ScratchUp(ScratchKey),
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
