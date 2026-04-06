//! emitter: adapters that turn `InputEvent` into runtime events.

use bevy::prelude::*;
use common::{InputEvent, LaneInputEvent};
fn keycode_to_lane(key: KeyCode) -> Option<usize> {
    use KeyCode::*;
    Some(match key {
        KeyS => 0,
        KeyD => 1,
        KeyF => 2,
        Space => 3,
        KeyJ => 4,
        KeyK => 5,
        KeyL => 6,
        ShiftLeft | ShiftRight => 7,
        _ => return None,
    })
}

pub fn input_events_to_lane_events(
    mut input_reader: MessageReader<InputEvent>,
    mut lane_writer: MessageWriter<LaneInputEvent>,
) {
    for ev in input_reader.read() {
        match ev {
            InputEvent::KeyDown(key) => {
                if let Some(lane) = keycode_to_lane(*key) {
                    lane_writer.write(LaneInputEvent { lane_index: lane, pressed: true });
                }
            }
            InputEvent::KeyUp(key) => {
                if let Some(lane) = keycode_to_lane(*key) {
                    lane_writer.write(LaneInputEvent { lane_index: lane, pressed: false });
                }
            }
            _ => {}
        }
    }
}

// Gamepad mapping (moved here so emitter produces lane events from gamepad too)
pub fn emit_gamepad_button_lane_input(
    gamepad_q: Query<&bevy::input::gamepad::Gamepad>,
    mut writer: MessageWriter<LaneInputEvent>,
) {
    use bevy::input::gamepad::GamepadButton;
    for gp in &gamepad_q {
        let map = [
            (GamepadButton::West, 0usize),
            (GamepadButton::LeftTrigger, 1usize),
            (GamepadButton::LeftThumb, 2usize),
            (GamepadButton::South, 3usize),
            (GamepadButton::North, 4usize),
            (GamepadButton::RightTrigger, 5usize),
            (GamepadButton::East, 6usize),
            (GamepadButton::RightThumb, 7usize),
        ];
        for (btn, lane_index) in map {
            if gp.just_pressed(btn) {
                writer.write(LaneInputEvent { lane_index, pressed: true });
            }
            if gp.just_released(btn) {
                writer.write(LaneInputEvent { lane_index, pressed: false });
            }
        }
    }
}

