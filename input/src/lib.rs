//! input: key polling and conversion to `common::InputEvent`.

use bevy::prelude::*;
use common::{InputEvent, StateId};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ActionBinding {
    Confirm,
    Cancel,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PlayBinding {
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
}

#[derive(Event, Message, Clone, Copy, Debug, PartialEq, Eq)]
pub enum NormalizedInputEvent {
    PlayKeyDown(PlayBinding),
    PlayKeyUp(PlayBinding),
    ScratchDown,
    ScratchUp,
    Confirm,
    Cancel,
}

impl InputEvent for NormalizedInputEvent {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum KeyBinding {
    Action(ActionBinding),
    Play(PlayBinding),
    Scratch,
}

#[derive(Debug, Default)]
struct StateBindings {
    bindings_by_key: HashMap<KeyCode, Vec<KeyBinding>>,
}

impl StateBindings {
    pub fn keys(&self) -> impl Iterator<Item = KeyCode> + '_ {
        self.bindings_by_key.keys().copied()
    }

    fn bind(&mut self, key: KeyCode, binding: KeyBinding) {
        self.bindings_by_key.entry(key).or_default().push(binding);
    }

    pub fn pressed_events(&self, key: KeyCode) -> Vec<NormalizedInputEvent> {
        let mut out = Vec::new();
        if let Some(bindings) = self.bindings_by_key.get(&key) {
            for binding in bindings {
                match binding {
                    KeyBinding::Action(ActionBinding::Confirm) => {
                        out.push(NormalizedInputEvent::Confirm)
                    }
                    KeyBinding::Action(ActionBinding::Cancel) => out.push(NormalizedInputEvent::Cancel),
                    KeyBinding::Play(play_key) => out.push(NormalizedInputEvent::PlayKeyDown(*play_key)),
                    KeyBinding::Scratch => out.push(NormalizedInputEvent::ScratchDown),
                }
            }
        }
        out
    }

    pub fn released_events(&self, key: KeyCode) -> Vec<NormalizedInputEvent> {
        let mut out = Vec::new();
        if let Some(bindings) = self.bindings_by_key.get(&key) {
            for binding in bindings {
                match binding {
                    KeyBinding::Play(play_key) => out.push(NormalizedInputEvent::PlayKeyUp(*play_key)),
                    KeyBinding::Scratch => out.push(NormalizedInputEvent::ScratchUp),
                    KeyBinding::Action(_) => {}
                }
            }
        }
        out
    }
}

const DEFAULT_BINDINGS_TOML: &str = r#"
[playing.play_keys]
S = "Key1"
D = "Key2"
F = "Key3"
J = "Key4"
K = "Key5"
L = "Key6"
ShiftLeft = "Key7"

[playing.scratch_keys]
Space = "Scratch"

[playing.actions]
Enter = "Confirm"
Escape = "Cancel"

[title.actions]
Enter = "Confirm"
Escape = "Cancel"

[result.scratch_keys]
Space = "Scratch"

[result.actions]
Enter = "Confirm"
Escape = "Cancel"
"#;

#[derive(Resource, Debug, Default)]
pub struct Bindings {
    title: StateBindings,
    playing: StateBindings,
    result: StateBindings,
}

impl Bindings {
    /// Returns a bindings set with sensible defaults.
    pub fn with_defaults() -> Self {
        Self::from_toml_str(DEFAULT_BINDINGS_TOML).expect("default bindings TOML must be valid")
    }

    fn state_bindings(&self, id: StateId) -> &StateBindings {
        match id {
            StateId::Title => &self.title,
            StateId::Playing => &self.playing,
            StateId::Result => &self.result,
        }
    }

    fn state_bindings_mut(&mut self, id: StateId) -> &mut StateBindings {
        match id {
            StateId::Title => &mut self.title,
            StateId::Playing => &mut self.playing,
            StateId::Result => &mut self.result,
        }
    }
}

impl Bindings {
    /// Attempt to load bindings from a TOML file at `path`.
    /// Expected format:
    /// [playing.play_keys]
    /// S = "Key1"
    ///
    /// [playing.scratch_keys]
    /// Space = "Scratch"
    ///
    /// [playing.actions]
    /// Enter = "Confirm"
    /// Escape = "Cancel"
    pub fn from_file<P: AsRef<std::path::Path>>(
        path: P,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let s = std::fs::read_to_string(path)?;
        Self::from_toml_str(&s)
    }

    fn from_toml_str(s: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let v: toml::Value = toml::from_str(&s)?;
        let mut bindings = Self::default();

        parse_state_section(&v, "title", "actions", |key, mapped| {
            if let Some(action) = parse_input_action(mapped) {
                bindings
                    .state_bindings_mut(StateId::Title)
                    .bind(key, KeyBinding::Action(action));
            } else {
                eprintln!("unknown action '{}', skipping", mapped);
            }
        });

        parse_state_section(&v, "playing", "actions", |key, mapped| {
            if let Some(action) = parse_input_action(mapped) {
                bindings
                    .state_bindings_mut(StateId::Playing)
                    .bind(key, KeyBinding::Action(action));
            } else {
                eprintln!("unknown action '{}', skipping", mapped);
            }
        });

        parse_state_section(&v, "result", "actions", |key, mapped| {
            if let Some(action) = parse_input_action(mapped) {
                bindings
                    .state_bindings_mut(StateId::Result)
                    .bind(key, KeyBinding::Action(action));
            } else {
                eprintln!("unknown action '{}', skipping", mapped);
            }
        });

        parse_state_section(&v, "playing", "play_keys", |key, mapped| {
            if let Some(play_key) = parse_play_key(mapped) {
                bindings
                    .state_bindings_mut(StateId::Playing)
                    .bind(key, KeyBinding::Play(play_key));
            } else {
                eprintln!("unknown play key '{}', skipping", mapped);
            }
        });

        parse_state_section(&v, "playing", "scratch_keys", |key, mapped| {
            if parse_scratch_key(mapped) {
                bindings
                    .state_bindings_mut(StateId::Playing)
                    .bind(key, KeyBinding::Scratch);
            } else {
                eprintln!("unknown scratch key '{}', skipping", mapped);
            }
        });

        parse_state_section(&v, "result", "scratch_keys", |key, mapped| {
            if parse_scratch_key(mapped) {
                bindings
                    .state_bindings_mut(StateId::Result)
                    .bind(key, KeyBinding::Scratch);
            } else {
                eprintln!("unknown scratch key '{}', skipping", mapped);
            }
        });

        Ok(bindings)
    }
}

fn parse_state_section<F>(v: &toml::Value, state: &str, section: &str, mut on_entry: F)
where
    F: FnMut(KeyCode, &str),
{
    let Some(toml::Value::Table(state_tbl)) = v.get(state) else {
        return;
    };
    let Some(toml::Value::Table(tbl)) = state_tbl.get(section) else {
        return;
    };
    for (k, val) in tbl {
        let Some(keycode) = parse_keycode(k) else {
            eprintln!("unknown key '{}', skipping", k);
            continue;
        };
        let toml::Value::String(mapped) = val else {
            eprintln!("invalid mapped value for key '{}', skipping", k);
            continue;
        };
        on_entry(keycode, mapped);
    }
}

fn parse_input_action(s: &str) -> Option<ActionBinding> {
    match s {
        "Confirm" | "confirm" => Some(ActionBinding::Confirm),
        "Cancel" | "cancel" => Some(ActionBinding::Cancel),
        _ => None,
    }
}

fn parse_play_key(s: &str) -> Option<PlayBinding> {
    match s {
        "Key1" | "1" => Some(PlayBinding::Key1),
        "Key2" | "2" => Some(PlayBinding::Key2),
        "Key3" | "3" => Some(PlayBinding::Key3),
        "Key4" | "4" => Some(PlayBinding::Key4),
        "Key5" | "5" => Some(PlayBinding::Key5),
        "Key6" | "6" => Some(PlayBinding::Key6),
        "Key7" | "7" => Some(PlayBinding::Key7),
        _ => None,
    }
}

fn parse_scratch_key(s: &str) -> bool {
    match s {
        "Scratch" | "scratch" => true,
        _ => false,
    }
}

fn parse_keycode(s: &str) -> Option<KeyCode> {
    use bevy::prelude::KeyCode::*;
    match s.trim() {
        "KeyS" | "S" => Some(KeyS),
        "KeyD" | "D" => Some(KeyD),
        "KeyF" | "F" => Some(KeyF),
        "Space" | " " => Some(Space),
        "KeyJ" | "J" => Some(KeyJ),
        "KeyK" | "K" => Some(KeyK),
        "KeyL" | "L" => Some(KeyL),
        "ShiftLeft" | "LeftShift" | "Shift" => Some(ShiftLeft),
        "ShiftRight" | "RightShift" => Some(ShiftRight),
        "Enter" => Some(Enter),
        "Escape" | "Esc" => Some(Escape),
        "Tab" => Some(Tab),
        other => {
            // Try to parse single character like "a" -> KeyA if needed
            let up = other.to_uppercase();
            if up.len() == 1 {
                match up.as_str() {
                    "A" => Some(KeyA),
                    "B" => Some(KeyB),
                    "C" => Some(KeyC),
                    "D" => Some(KeyD),
                    "E" => Some(KeyE),
                    "F" => Some(KeyF),
                    "G" => Some(KeyG),
                    "H" => Some(KeyH),
                    "I" => Some(KeyI),
                    "J" => Some(KeyJ),
                    "K" => Some(KeyK),
                    "L" => Some(KeyL),
                    "M" => Some(KeyM),
                    "N" => Some(KeyN),
                    "O" => Some(KeyO),
                    "P" => Some(KeyP),
                    "Q" => Some(KeyQ),
                    "R" => Some(KeyR),
                    "S" => Some(KeyS),
                    "T" => Some(KeyT),
                    "U" => Some(KeyU),
                    "V" => Some(KeyV),
                    "W" => Some(KeyW),
                    "X" => Some(KeyX),
                    "Y" => Some(KeyY),
                    "Z" => Some(KeyZ),
                    _ => None,
                }
            } else {
                None
            }
        }
    }
}

fn poll_key_events_for_state(
    state: StateId,
    keys: Res<ButtonInput<KeyCode>>,
    mut ev_writer: MessageWriter<NormalizedInputEvent>,
    bindings: Res<Bindings>,
) {
    let state_bindings = bindings.state_bindings(state);
    let mut tracked = HashSet::new();
    for k in state_bindings.keys() {
        tracked.insert(k);
    }

    for key in tracked {
        if keys.just_pressed(key) {
            for ev in state_bindings.pressed_events(key) {
                ev_writer.write(ev);
            }
        }
        if keys.just_released(key) {
            for ev in state_bindings.released_events(key) {
                ev_writer.write(ev);
            }
        }
    }
}

pub fn poll_title_key_events(
    keys: Res<ButtonInput<KeyCode>>,
    ev_writer: MessageWriter<NormalizedInputEvent>,
    bindings: Res<Bindings>,
) {
    poll_key_events_for_state(StateId::Title, keys, ev_writer, bindings);
}

pub fn poll_playing_key_events(
    keys: Res<ButtonInput<KeyCode>>,
    ev_writer: MessageWriter<NormalizedInputEvent>,
    bindings: Res<Bindings>,
) {
    poll_key_events_for_state(StateId::Playing, keys, ev_writer, bindings);
}

pub fn poll_result_key_events(
    keys: Res<ButtonInput<KeyCode>>,
    ev_writer: MessageWriter<NormalizedInputEvent>,
    bindings: Res<Bindings>,
) {
    poll_key_events_for_state(StateId::Result, keys, ev_writer, bindings);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bindings_from_file_parses_valid_bindings() -> Result<(), Box<dyn std::error::Error>> {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "wakamore_bindings_test_{}.toml",
            std::process::id()
        ));
        let toml = r#"
[title.actions]
Enter = "Confirm"
Escape = "Cancel"

[playing.actions]
Enter = "Confirm"
Escape = "Cancel"

[playing.play_keys]
S = "Key1"
J = "Key4"

[playing.scratch_keys]
Space = "Scratch"

[result.actions]
Enter = "Confirm"

[result.scratch_keys]
Space = "Scratch"
"#;
        std::fs::write(&path, toml)?;

        let b = Bindings::from_file(&path)?;
        let title_enter = b
            .state_bindings(StateId::Title)
            .pressed_events(bevy::prelude::KeyCode::Enter);
        assert_eq!(title_enter, vec![NormalizedInputEvent::Confirm]);

        let playing_s_pressed = b
            .state_bindings(StateId::Playing)
            .pressed_events(bevy::prelude::KeyCode::KeyS);
        assert_eq!(playing_s_pressed, vec![NormalizedInputEvent::PlayKeyDown(PlayBinding::Key1)]);

        let playing_s_released = b
            .state_bindings(StateId::Playing)
            .released_events(bevy::prelude::KeyCode::KeyS);
        assert_eq!(playing_s_released, vec![NormalizedInputEvent::PlayKeyUp(PlayBinding::Key1)]);

        let result_space_pressed = b
            .state_bindings(StateId::Result)
            .pressed_events(bevy::prelude::KeyCode::Space);
        assert_eq!(result_space_pressed, vec![NormalizedInputEvent::ScratchDown]);

        let result_space_released = b
            .state_bindings(StateId::Result)
            .released_events(bevy::prelude::KeyCode::Space);
        assert_eq!(result_space_released, vec![NormalizedInputEvent::ScratchUp]);

        let _ = std::fs::remove_file(&path);
        Ok(())
    }

    #[test]
    fn parse_keycode_accepts_single_letters_and_known_names() {
        assert_eq!(parse_keycode("A"), Some(bevy::prelude::KeyCode::KeyA));
        assert_eq!(parse_keycode("a"), Some(bevy::prelude::KeyCode::KeyA));
        assert_eq!(parse_keycode("Space"), Some(bevy::prelude::KeyCode::Space));
        assert_eq!(parse_keycode("Enter"), Some(bevy::prelude::KeyCode::Enter));
        assert_eq!(parse_keycode("KeyS"), Some(bevy::prelude::KeyCode::KeyS));
        assert_eq!(
            parse_keycode("Shift"),
            Some(bevy::prelude::KeyCode::ShiftLeft)
        );
        assert_eq!(parse_keycode("UnknownKey"), None);
    }
}
