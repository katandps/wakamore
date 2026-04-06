use crate::component::note::LaneKeyIndicator;
use crate::event::note::LaneInputEvent;
use bevy::prelude::*;

pub fn apply_lane_input_visuals(
    mut input_event_reader: MessageReader<LaneInputEvent>,
    mut indicator_q: Query<(&LaneKeyIndicator, &mut BackgroundColor)>,
) {
    for event in input_event_reader.read() {
        for (indicator, mut bg) in &mut indicator_q {
            if indicator.index != event.lane_index {
                continue;
            }
            if event.pressed {
                bg.0 = Color::srgb(1.0, 0.82, 0.25);
            } else {
                bg.0 = Color::srgb(0.22, 0.24, 0.28);
            }
        }
    }
}
