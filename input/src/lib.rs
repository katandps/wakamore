//! input: key polling and conversion to `common::InputEvent`.

use bevy::prelude::*;
use common::{InputAction, InputEvent, KeyToAction, KeyToActionResource};
use std::collections::{HashMap, HashSet};

#[derive(Resource, Debug, Default)]
pub struct Bindings(pub HashMap<KeyCode, InputAction>);

impl Bindings {
    /// Returns a bindings set with sensible defaults.
    pub fn with_defaults() -> Self {
        use KeyCode::*;
        let mut map = HashMap::new();
        map.insert(Space, InputAction::Confirm);
        map.insert(Enter, InputAction::Confirm);
        map.insert(Escape, InputAction::Cancel);
        Self(map)
    }

    pub fn keys(&self) -> impl Iterator<Item = KeyCode> + '_ {
        self.0.keys().copied()
    }

    pub fn map_key(&self, key: KeyCode) -> Option<InputAction> {
        self.0.get(&key).cloned()
    }
}

impl KeyToAction for Bindings {
    fn key_to_action(&self, key: KeyCode) -> Option<InputAction> {
        self.map_key(key)
    }

    fn bound_keys(&self) -> Vec<KeyCode> {
        self.0.keys().copied().collect()
    }
}

impl Bindings {
    /// Attempt to load bindings from a TOML file at `path`.
    /// Expected format:
    /// [bindings]
    /// Space = "Confirm"
    /// Enter = "Confirm"
    pub fn from_file<P: AsRef<std::path::Path>>(
        path: P,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let s = std::fs::read_to_string(path)?;
        let v: toml::Value = toml::from_str(&s)?;
        let mut map = HashMap::new();
        if let Some(bindings_table) = v.get("bindings") {
            if let toml::Value::Table(tbl) = bindings_table {
                for (k, val) in tbl {
                    if let toml::Value::String(action_str) = val {
                        let action = match action_str.as_str() {
                            "Confirm" | "confirm" => InputAction::Confirm,
                            "Cancel" | "cancel" => InputAction::Cancel,
                            other => {
                                eprintln!("unknown action '{}', skipping", other);
                                continue;
                            }
                        };
                        if let Some(keycode) = parse_keycode(k) {
                            map.insert(keycode, action);
                        } else {
                            eprintln!("unknown key '{}', skipping", k);
                        }
                    } else {
                        eprintln!("invalid action value for key '{}', skipping", k);
                    }
                }
            }
        }
        Ok(Self(map))
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
    mut ev_writer: MessageWriter<InputEvent>,
    bindings: Option<Res<KeyToActionResource>>,
) {
    use KeyCode::*;

    const TRACKED_KEYS: [KeyCode; 9] = [
        KeyS, KeyD, KeyF, Space, KeyJ, KeyK, KeyL, ShiftLeft, ShiftRight,
    ];

    // build union of tracked keys and keys present in bindings
    let mut tracked = HashSet::new();
    for k in TRACKED_KEYS {
        tracked.insert(k);
    }
    if let Some(b) = bindings.as_ref() {
        for k in b.0.bound_keys() {
            tracked.insert(k);
        }
    }

    for key in tracked {
        if keys.just_pressed(key) {
            ev_writer.write(InputEvent::KeyDown(key));
            if let Some(b) = bindings.as_ref() {
                if let Some(action) = b.0.key_to_action(key) {
                    ev_writer.write(InputEvent::Action(action));
                }
            }
        }
        if keys.just_released(key) {
            ev_writer.write(InputEvent::KeyUp(key));
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
[bindings]
Space = "Confirm"
Enter = "Confirm"
A = "Cancel"
"#;
        std::fs::write(&path, toml)?;

        let b = Bindings::from_file(&path)?;
        assert_eq!(
            b.map_key(bevy::prelude::KeyCode::Space),
            Some(InputAction::Confirm)
        );
        assert_eq!(
            b.map_key(bevy::prelude::KeyCode::Enter),
            Some(InputAction::Confirm)
        );
        assert_eq!(
            b.map_key(bevy::prelude::KeyCode::KeyA),
            Some(InputAction::Cancel)
        );

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
