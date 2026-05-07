//! input: key polling and conversion to `common::InputEvent`.

use bevy::prelude::*;
use common::{PlayBinding, PlayingInputEvent, ResultInputEvent, TitleInputEvent};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ActionBinding {
    Confirm,
    Cancel,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PlayingActionBinding {
    Abort,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum KeyBinding {
    Action(ActionBinding),
    PlayingAction(PlayingActionBinding),
    Play(PlayBinding),
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

    pub fn pressed_bindings(&self, key: KeyCode) -> Vec<KeyBinding> {
        let mut out = Vec::new();
        if let Some(bindings) = self.bindings_by_key.get(&key) {
            out.extend(bindings.iter().copied());
        }
        out
    }

    pub fn released_bindings(&self, key: KeyCode) -> Vec<KeyBinding> {
        let mut out = Vec::new();
        if let Some(bindings) = self.bindings_by_key.get(&key) {
            out.extend(bindings.iter().copied());
        }
        out
    }
}

const DEFAULT_BINDINGS_TOML: &str = r#"
[playing.play_keys]
Key1 = "S"
Key2 = "D"
Key3 = "F"
Key4 = "Space"
Key5 = "J"
Key6 = "K"
Key7 = "L"
ScratchUp = "RShift"
ScratchDown = "LShift"

[playing.actions]
Abort = "R"

[title.actions]
Confirm = "Enter"
Cancel = "Escape"

[result.actions]
Confirm = "Enter"
Cancel = "Escape"
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

    fn title_bindings(&self) -> &StateBindings {
        &self.title
    }

    fn title_bindings_mut(&mut self) -> &mut StateBindings {
        &mut self.title
    }

    fn playing_bindings(&self) -> &StateBindings {
        &self.playing
    }

    fn playing_bindings_mut(&mut self) -> &mut StateBindings {
        &mut self.playing
    }

    fn result_bindings(&self) -> &StateBindings {
        &self.result
    }

    fn result_bindings_mut(&mut self) -> &mut StateBindings {
        &mut self.result
    }
}

impl Bindings {
    /// Attempt to load bindings from a TOML file at `path`.
    /// Expected format:
    /// [playing.play_keys]
    /// Key1 = "S"
    /// ScratchUp = "Space"
    /// ScratchDown = "RShift"
    ///
    /// [playing.actions]
    /// Abort = "R"
    ///
    /// [result.actions]
    /// Confirm = "Enter"
    /// Cancel = "Escape"
    pub fn from_file<P: AsRef<std::path::Path>>(
        path: P,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let s = std::fs::read_to_string(path)?;
        Self::from_toml_str(&s)
    }

    fn from_toml_str(s: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let v: toml::Value = toml::from_str(&s)?;
        let mut bindings = Self::default();

        parse_state_event_section(&v, "title", "actions", |event, key| {
            if let Some(action) = parse_input_action(event) {
                bindings
                    .title_bindings_mut()
                    .bind(key, KeyBinding::Action(action));
            } else {
                eprintln!("unknown action '{}', skipping", event);
            }
        });

        parse_state_event_section(&v, "result", "actions", |event, key| {
            if let Some(action) = parse_input_action(event) {
                bindings
                    .result_bindings_mut()
                    .bind(key, KeyBinding::Action(action));
            } else {
                eprintln!("unknown action '{}', skipping", event);
            }
        });

        parse_state_event_section(&v, "playing", "play_keys", |event, key| {
            if let Some(play_key) = parse_play_key(event) {
                bindings
                    .playing_bindings_mut()
                    .bind(key, KeyBinding::Play(play_key));
            } else {
                eprintln!("unknown play key '{}', skipping", event);
            }
        });

        parse_state_event_section(&v, "playing", "actions", |event, key| {
            if let Some(action) = parse_playing_action(event) {
                bindings
                    .playing_bindings_mut()
                    .bind(key, KeyBinding::PlayingAction(action));
            } else {
                eprintln!("unknown playing action '{}', skipping", event);
            }
        });

        Ok(bindings)
    }
}

fn parse_state_event_section<F>(v: &toml::Value, state: &str, section: &str, mut on_entry: F)
where
    F: FnMut(&str, KeyCode),
{
    let Some(toml::Value::Table(state_tbl)) = v.get(state) else {
        return;
    };
    let Some(toml::Value::Table(tbl)) = state_tbl.get(section) else {
        return;
    };
    for (event, val) in tbl {
        let toml::Value::String(key_name) = val else {
            eprintln!("invalid key value for event '{}', skipping", event);
            continue;
        };
        let Some(keycode) = parse_keycode(key_name) else {
            eprintln!("unknown key '{}', skipping", key_name);
            continue;
        };
        on_entry(event, keycode);
    }
}

fn parse_input_action(s: &str) -> Option<ActionBinding> {
    match s {
        "Confirm" | "confirm" => Some(ActionBinding::Confirm),
        "Cancel" | "cancel" => Some(ActionBinding::Cancel),
        _ => None,
    }
}

fn parse_playing_action(s: &str) -> Option<PlayingActionBinding> {
    match s {
        "Abort" => Some(PlayingActionBinding::Abort),
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
        "ScratchUp" | "scratch_up" => Some(PlayBinding::ScratchUp),
        "ScratchDown" | "scratch_down" => Some(PlayBinding::ScratchDown),
        _ => None,
    }
}

fn parse_keycode(s: &str) -> Option<KeyCode> {
    use bevy::prelude::KeyCode::*;
    match s.trim() {
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
        "Space" => Some(Space),
        "LShift" => Some(ShiftLeft),
        "RShift" => Some(ShiftRight),
        "Enter" => Some(Enter),
        "Escape" => Some(Escape),
        "Tab" => Some(Tab),
        _ => None,
    }
}

fn poll_key_events_for_bindings(
    state_bindings: &StateBindings,
    keys: Res<ButtonInput<KeyCode>>,
    mut on_event: impl FnMut(KeyBinding, bool),
) {
    for key in state_bindings.keys() {
        if keys.just_pressed(key) {
            for binding in state_bindings.pressed_bindings(key) {
                on_event(binding, true);
            }
        }
        if keys.just_released(key) {
            for binding in state_bindings.released_bindings(key) {
                on_event(binding, false);
            }
        }
    }
}

pub fn poll_title_key_events(
    keys: Res<ButtonInput<KeyCode>>,
    mut ev_writer: MessageWriter<TitleInputEvent>,
    bindings: Res<Bindings>,
) {
    poll_key_events_for_bindings(
        bindings.title_bindings(),
        keys,
        |binding, pressed| {
            if !pressed {
                return;
            }
            match binding {
                KeyBinding::Action(ActionBinding::Confirm) => {
                    ev_writer.write(TitleInputEvent::Confirm);
                }
                KeyBinding::Action(ActionBinding::Cancel) => {
                    ev_writer.write(TitleInputEvent::Cancel);
                }
                _ => {}
            }
        },
    );
}

pub fn poll_playing_key_events(
    keys: Res<ButtonInput<KeyCode>>,
    mut ev_writer: MessageWriter<PlayingInputEvent>,
    bindings: Res<Bindings>,
) {
    poll_key_events_for_bindings(
        bindings.playing_bindings(),
        keys,
        |binding, pressed| {
            match (binding, pressed) {
                (KeyBinding::Play(play_key), true) => {
                    ev_writer.write(PlayingInputEvent::PlayKeyDown(play_key));
                }
                (KeyBinding::Play(play_key), false) => {
                    ev_writer.write(PlayingInputEvent::PlayKeyUp(play_key));
                }
                (KeyBinding::PlayingAction(PlayingActionBinding::Abort), true) => {
                    ev_writer.write(PlayingInputEvent::Abort);
                }
                (KeyBinding::PlayingAction(_), false) => {}
                (KeyBinding::Action(_), _) => {}
            }
        },
    );
}

pub fn poll_result_key_events(
    keys: Res<ButtonInput<KeyCode>>,
    mut ev_writer: MessageWriter<ResultInputEvent>,
    bindings: Res<Bindings>,
) {
    poll_key_events_for_bindings(
        bindings.result_bindings(),
        keys,
        |binding, pressed| {
            if !pressed {
                return;
            }
            match binding {
                KeyBinding::Action(ActionBinding::Confirm) => {
                    ev_writer.write(ResultInputEvent::Confirm);
                }
                KeyBinding::Action(ActionBinding::Cancel) => {
                    ev_writer.write(ResultInputEvent::Cancel);
                }
                _ => {}
            }
        },
    );
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
    Confirm = "Enter"
    Cancel = "Escape"

[playing.play_keys]
    Key1 = "S"
    Key4 = "J"
    ScratchUp = "Space"
    ScratchDown = "RShift"

[playing.actions]
    Abort = "R"

[result.actions]
    Confirm = "Enter"
    Cancel = "Escape"
"#;
        std::fs::write(&path, toml)?;

        let b = Bindings::from_file(&path)?;
        let title_enter = b
            .title_bindings()
            .pressed_bindings(bevy::prelude::KeyCode::Enter);
        assert_eq!(title_enter, vec![KeyBinding::Action(ActionBinding::Confirm)]);

        let playing_s_pressed = b
            .playing_bindings()
            .pressed_bindings(bevy::prelude::KeyCode::KeyS);
        assert_eq!(playing_s_pressed, vec![KeyBinding::Play(PlayBinding::Key1)]);

        let playing_force_result_pressed = b
            .playing_bindings()
            .pressed_bindings(bevy::prelude::KeyCode::KeyR);
        assert_eq!(
            playing_force_result_pressed,
            vec![KeyBinding::PlayingAction(PlayingActionBinding::Abort)]
        );

        let playing_s_released = b
            .playing_bindings()
            .released_bindings(bevy::prelude::KeyCode::KeyS);
        assert_eq!(playing_s_released, vec![KeyBinding::Play(PlayBinding::Key1)]);

        let result_space_pressed = b
            .result_bindings()
            .pressed_bindings(bevy::prelude::KeyCode::Enter);
        assert_eq!(result_space_pressed, vec![KeyBinding::Action(ActionBinding::Confirm)]);

        let result_space_released = b
            .result_bindings()
            .released_bindings(bevy::prelude::KeyCode::Enter);
        assert_eq!(
            result_space_released,
            vec![KeyBinding::Action(ActionBinding::Confirm)]
        );

        let _ = std::fs::remove_file(&path);
        Ok(())
    }

    #[test]
    fn parse_keycode_accepts_single_letters_and_known_names() {
        assert_eq!(parse_keycode("A"), Some(bevy::prelude::KeyCode::KeyA));
        assert_eq!(parse_keycode("a"), None);
        assert_eq!(parse_keycode("Space"), Some(bevy::prelude::KeyCode::Space));
        assert_eq!(parse_keycode(" "), None);
        assert_eq!(parse_keycode("Enter"), Some(bevy::prelude::KeyCode::Enter));
        assert_eq!(parse_keycode("KeyS"), None);
        assert_eq!(parse_keycode("Shift"), None);
        assert_eq!(
            parse_keycode("LShift"),
            Some(bevy::prelude::KeyCode::ShiftLeft)
        );
        assert_eq!(
            parse_keycode("RShift"),
            Some(bevy::prelude::KeyCode::ShiftRight)
        );
        assert_eq!(parse_keycode("Esc"), None);
        assert_eq!(parse_keycode("UnknownKey"), None);
    }
}
