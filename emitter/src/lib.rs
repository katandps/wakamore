//! emitter: adapters that turn `InputEvent` into runtime events.

use bevy::prelude::*;
use common::{InputEvent, LaneInputEvent, LastRawByLane, PlayBinding, PlayingInputEvent, RawInput};

pub trait LaneEventSource: InputEvent {
    fn lane_input(&self) -> Option<(usize, bool)>;
}

impl LaneEventSource for PlayingInputEvent {
    fn lane_input(&self) -> Option<(usize, bool)> {
        match self {
            PlayingInputEvent::PlayKeyDown(k) => Some((play_key_to_lane(*k), true)),
            PlayingInputEvent::PlayKeyUp(k) => Some((play_key_to_lane(*k), false)),
            PlayingInputEvent::Abort => None,
        }
    }
}

fn play_key_to_lane(key: PlayBinding) -> usize {
    match key {
        PlayBinding::Key1 => 0,
        PlayBinding::Key2 => 1,
        PlayBinding::Key3 => 2,
        PlayBinding::Key4 => 3,
        PlayBinding::Key5 => 4,
        PlayBinding::Key6 => 5,
        PlayBinding::Key7 => 6,
        PlayBinding::ScratchUp | PlayBinding::ScratchDown => 7,
    }
}

pub fn input_events_to_lane_events<E: LaneEventSource>(
    mut input_reader: MessageReader<E>,
    mut lane_writer: MessageWriter<LaneInputEvent>,
) {
    for ev in input_reader.read() {
        if let Some((lane_index, pressed)) = ev.lane_input() {
            lane_writer.write(LaneInputEvent {
                lane_index,
                pressed,
                raw: None,
            });
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

    #[derive(Event, Message, Clone)]
    enum TestInputEvent {
        Lane0Down,
        Lane0Up,
    }

    impl InputEvent for TestInputEvent {}

    impl LaneEventSource for TestInputEvent {
        fn lane_input(&self) -> Option<(usize, bool)> {
            match self {
                TestInputEvent::Lane0Down => Some((0, true)),
                TestInputEvent::Lane0Up => Some((0, false)),
            }
        }
    }

    #[derive(Resource, Default)]
    struct Out(pub Vec<LaneInputEvent>);

    fn emit_input(mut writer: MessageWriter<TestInputEvent>) {
        writer.write(TestInputEvent::Lane0Down);
        writer.write(TestInputEvent::Lane0Up);
    }

    fn collect(mut reader: MessageReader<LaneInputEvent>, mut out: ResMut<Out>) {
        for ev in reader.read() {
            out.0.push(ev.clone());
        }
    }

    #[test]
    fn input_events_convert_to_lane_events() {
        let mut app = App::new();
        app.add_message::<TestInputEvent>();
        app.add_message::<LaneInputEvent>();
        app.init_resource::<Out>();
        app.add_systems(Startup, emit_input);
        app.add_systems(
            Update,
            (input_events_to_lane_events::<TestInputEvent>, collect),
        );

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
