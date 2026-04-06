use bevy::prelude::*;
use crate::component::note::LaneKeyIndicator;
use crate::component::{lane_is_pressed, lane_is_just_pressed};
use crate::resource::note::LaneInputState;
use crate::event::note::LaneInputEvent;

pub fn collect_lane_input_state(
    keys: Res<ButtonInput<KeyCode>>,
    mut input_state: ResMut<LaneInputState>,
) {
    for lane_index in 0..input_state.pressed.len() {
        input_state.pressed[lane_index] = lane_is_pressed(&keys, lane_index);
        input_state.just_pressed[lane_index] = lane_is_just_pressed(&keys, lane_index);
    }
}

pub fn emit_lane_input_events(
    input_state: Res<LaneInputState>,
    mut input_event_writer: MessageWriter<LaneInputEvent>,
) {
    for lane_index in 0..input_state.pressed.len() {
        input_event_writer.write(LaneInputEvent { lane_index, pressed: input_state.pressed[lane_index] });
    }
}

pub fn apply_lane_input_visuals(
    mut input_event_reader: MessageReader<LaneInputEvent>,
    mut indicator_q: Query<(&LaneKeyIndicator, &mut BackgroundColor)>,
) {
    for event in input_event_reader.read() {
        for (indicator, mut bg) in &mut indicator_q {
            if indicator.index != event.lane_index { continue; }
            if event.pressed { bg.0 = Color::srgb(1.0, 0.82, 0.25); } else { bg.0 = Color::srgb(0.22, 0.24, 0.28); }
        }
    }
}
