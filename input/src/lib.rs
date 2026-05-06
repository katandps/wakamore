//! input: key polling and conversion to `common::InputEvent`.

use bevy::prelude::*;
use common::InputEvent;
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

impl Bindings {
    pub fn pressed_events(&self, key: KeyCode) -> Vec<NormalizedInputEvent> {
        let mut out = Vec::new();
        if let Some(play_key) = self.play_key_for_key(key) {
            out.push(NormalizedInputEvent::PlayKeyDown(play_key));
        }
        if self.scratch_key_for_key(key) {
            out.push(NormalizedInputEvent::ScratchDown);
        }
        if let Some(action) = self.action_for_key(key) {
            match action {
                ActionBinding::Confirm => out.push(NormalizedInputEvent::Confirm),
                ActionBinding::Cancel => out.push(NormalizedInputEvent::Cancel),
            }
        }
        out
    }

    pub fn released_events(&self, key: KeyCode) -> Vec<NormalizedInputEvent> {
        let mut out = Vec::new();
        if let Some(play_key) = self.play_key_for_key(key) {
            out.push(NormalizedInputEvent::PlayKeyUp(play_key));
        }
        if self.scratch_key_for_key(key) {
            out.push(NormalizedInputEvent::ScratchUp);
        }
        out
    }
}

const DEFAULT_BINDINGS_TOML: &str = r#"
[play_keys]
S = "Key1"
D = "Key2"
F = "Key3"
J = "Key4"
K = "Key5"
L = "Key6"
ShiftLeft = "Key7"

[scratch_keys]
Space = "Scratch"

[actions]
Enter = "Confirm"
Escape = "Cancel"
"#;

#[derive(Resource, Debug, Default)]
pub struct Bindings {
    action_by_key: HashMap<KeyCode, ActionBinding>,
    play_key_by_key: HashMap<KeyCode, PlayBinding>,
    scratch_key_by_key: HashSet<KeyCode>,
}

impl Bindings {
    /// Returns a bindings set with sensible defaults.
    pub fn with_defaults() -> Self {
        Self::from_toml_str(DEFAULT_BINDINGS_TOML).expect("default bindings TOML must be valid")
    }

    pub fn keys(&self) -> impl Iterator<Item = KeyCode> + '_ {
        self.action_by_key
            .keys()
            .chain(self.play_key_by_key.keys())
            .chain(self.scratch_key_by_key.iter())
            .copied()
    }

    fn action_for_key(&self, key: KeyCode) -> Option<ActionBinding> {
        self.action_by_key.get(&key).cloned()
    }

    fn play_key_for_key(&self, key: KeyCode) -> Option<PlayBinding> {
        self.play_key_by_key.get(&key).copied()
    }

    fn scratch_key_for_key(&self, key: KeyCode) -> bool {
        self.scratch_key_by_key.contains(&key)
    }
}

impl Bindings {
    /// Attempt to load bindings from a TOML file at `path`.
    /// Expected format:
    /// [play_keys]
    /// S = "Key1"
    ///
    /// [scratch_keys]
    /// Space = "Scratch"
    ///
    /// [actions]
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

        parse_section(&v, "actions", |key, mapped| {
            if let Some(action) = parse_input_action(mapped) {
                bindings.action_by_key.insert(key, action);
            } else {
                eprintln!("unknown action '{}', skipping", mapped);
            }
        });

        parse_section(&v, "play_keys", |key, mapped| {
            if let Some(play_key) = parse_play_key(mapped) {
                bindings.play_key_by_key.insert(key, play_key);
            } else {
                eprintln!("unknown play key '{}', skipping", mapped);
            }
        });

        parse_section(&v, "scratch_keys", |key, mapped| {
            if parse_scratch_key(mapped) {
                bindings.scratch_key_by_key.insert(key);
            } else {
                eprintln!("unknown scratch key '{}', skipping", mapped);
            }
        });

        Ok(bindings)
    }
}

fn parse_section<F>(v: &toml::Value, name: &str, mut on_entry: F)
where
    F: FnMut(KeyCode, &str),
{
    let Some(toml::Value::Table(tbl)) = v.get(name) else {
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

pub fn poll_key_events(
    keys: Res<ButtonInput<KeyCode>>,
    mut ev_writer: MessageWriter<NormalizedInputEvent>,
    bindings: Res<Bindings>,
) {
    let mut tracked = HashSet::new();
    for k in bindings.keys() {
        tracked.insert(k);
    }

    for key in tracked {
        if keys.just_pressed(key) {
            for ev in bindings.pressed_events(key) {
                ev_writer.write(ev);
            }
        }
        if keys.just_released(key) {
            for ev in bindings.released_events(key) {
                ev_writer.write(ev);
            }
        }
    }
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
[actions]
Enter = "Confirm"
Escape = "Cancel"

[play_keys]
S = "Key1"
J = "Key4"

[scratch_keys]
Space = "Scratch"
"#;
        std::fs::write(&path, toml)?;

        let b = Bindings::from_file(&path)?;
        assert_eq!(
            b.action_for_key(bevy::prelude::KeyCode::Enter),
            Some(ActionBinding::Confirm)
        );
        assert_eq!(
            b.action_for_key(bevy::prelude::KeyCode::Escape),
            Some(ActionBinding::Cancel)
        );
        assert_eq!(
            b.play_key_for_key(bevy::prelude::KeyCode::KeyS),
            Some(PlayBinding::Key1)
        );
        assert_eq!(
            b.play_key_for_key(bevy::prelude::KeyCode::KeyJ),
            Some(PlayBinding::Key4)
        );
        assert!(b.scratch_key_for_key(bevy::prelude::KeyCode::Space));

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
