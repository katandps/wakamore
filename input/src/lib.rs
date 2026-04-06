//! input: key polling and conversion to `common::InputEvent`.

use bevy::prelude::*;
use common::InputEvent;

pub fn poll_key_events(keys: Res<ButtonInput<KeyCode>>, mut ev_writer: MessageWriter<InputEvent>) {
    use KeyCode::*;

    const TRACKED_KEYS: [KeyCode; 9] = [
        KeyS, KeyD, KeyF, Space, KeyJ, KeyK, KeyL, ShiftLeft, ShiftRight,
    ];

    for key in TRACKED_KEYS {
        if keys.just_pressed(key) {
            ev_writer.write(InputEvent::KeyDown(key));
        }
        if keys.just_released(key) {
            ev_writer.write(InputEvent::KeyUp(key));
        }
    }
}
