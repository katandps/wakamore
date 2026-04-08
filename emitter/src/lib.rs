//! emitter: adapters that turn `InputEvent` into runtime events.

use bevy::prelude::*;
use common::{InputEvent, LaneInputEvent, LastRawByLane, RawInput};
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
                    lane_writer.write(LaneInputEvent {
                        lane_index: lane,
                        pressed: true,
                        raw: Some(RawInput::Key(*key)),
                    });
                }
            }
            InputEvent::KeyUp(key) => {
                if let Some(lane) = keycode_to_lane(*key) {
                    lane_writer.write(LaneInputEvent {
                        lane_index: lane,
                        pressed: false,
                        raw: Some(RawInput::Key(*key)),
                    });
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
                writer.write(LaneInputEvent {
                    lane_index,
                    pressed: true,
                    raw: Some(RawInput::Gamepad(btn)),
                });
            }
            if gp.just_released(btn) {
                writer.write(LaneInputEvent {
                    lane_index,
                    pressed: false,
                    raw: Some(RawInput::Gamepad(btn)),
                });
            }
        }
    }
}

pub fn record_lane_raw_events(
    mut lane_reader: MessageReader<LaneInputEvent>,
    mut last_raw: ResMut<LastRawByLane>,
) {
    for ev in lane_reader.read() {
        if let Some(raw) = ev.raw.clone() {
            (*last_raw).0.insert(ev.lane_index, raw);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use common::InputEvent;

    #[derive(Resource, Default)]
    struct Out(pub Vec<LaneInputEvent>);

    fn emit_input(mut writer: MessageWriter<InputEvent>) {
        use KeyCode::*;
        writer.write(InputEvent::KeyDown(KeyS));
        writer.write(InputEvent::KeyUp(KeyS));
    }

    fn collect(mut reader: MessageReader<LaneInputEvent>, mut out: ResMut<Out>) {
        for ev in reader.read() {
            out.0.push(ev.clone());
        }
    }

    #[test]
    fn input_events_convert_to_lane_events() {
        let mut app = App::new();
        app.add_message::<InputEvent>();
        app.add_message::<LaneInputEvent>();
        app.init_resource::<Out>();
        app.add_systems(Startup, emit_input);
        app.add_systems(Update, (input_events_to_lane_events, collect));

        // Run update cycles to ensure startup and message propagation
        app.update();
        app.update();

        let out = app.world().resource::<Out>();
        assert_eq!(out.0.len(), 2);
        assert_eq!(out.0[0].lane_index, 0);
        assert!(out.0[0].pressed);
        assert_eq!(out.0[1].lane_index, 0);
        assert!(!out.0[1].pressed);
    }
}
