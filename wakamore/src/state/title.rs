use bevy::prelude::*;
use common::InputEvent;
use input::NormalizedInputEvent;

use crate::state::{AppState, TitleEntity};

#[derive(Event, Message, Clone, Copy, Debug, PartialEq, Eq)]
pub enum TitleInputEvent {
    Confirm,
    Cancel,
}

impl InputEvent for TitleInputEvent {}

pub fn map_title_input_events(
    mut in_reader: MessageReader<NormalizedInputEvent>,
    mut out_writer: MessageWriter<TitleInputEvent>,
) {
    for ev in in_reader.read() {
        match ev {
            NormalizedInputEvent::Confirm => {
                out_writer.write(TitleInputEvent::Confirm);
            }
            NormalizedInputEvent::Cancel => {
                out_writer.write(TitleInputEvent::Cancel);
            }
            _ => {}
        }
    }
}

pub fn update_title_input(
    mut input_reader: MessageReader<TitleInputEvent>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for ev in input_reader.read() {
        if matches!(ev, TitleInputEvent::Confirm) {
            next_state.set(AppState::Playing);
            break;
        }
    }
}

pub fn cleanup_title(mut commands: Commands, q: Query<Entity, With<TitleEntity>>) {
    for entity in &q {
        commands.entity(entity).despawn();
    }
}
