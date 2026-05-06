use bevy::prelude::*;
use common::InputEvent;
use emitter::LaneEventSource;
use input::{NormalizedInputEvent, PlayBinding};

use crate::component::GameplayEntity;
use crate::state::AppState;

#[derive(Event, Message, Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayingInputEvent {
    PlayKeyDown(PlayBinding),
    PlayKeyUp(PlayBinding),
    ScratchDown,
    ScratchUp,
    Confirm,
    Cancel,
}

impl InputEvent for PlayingInputEvent {}

pub fn map_playing_input_events(
    mut in_reader: MessageReader<NormalizedInputEvent>,
    mut out_writer: MessageWriter<PlayingInputEvent>,
) {
    for ev in in_reader.read() {
        match ev {
            NormalizedInputEvent::PlayKeyDown(k) => {
                out_writer.write(PlayingInputEvent::PlayKeyDown(*k));
            }
            NormalizedInputEvent::PlayKeyUp(k) => {
                out_writer.write(PlayingInputEvent::PlayKeyUp(*k));
            }
            NormalizedInputEvent::ScratchDown => {
                out_writer.write(PlayingInputEvent::ScratchDown);
            }
            NormalizedInputEvent::ScratchUp => {
                out_writer.write(PlayingInputEvent::ScratchUp);
            }
            NormalizedInputEvent::Confirm => {
                out_writer.write(PlayingInputEvent::Confirm);
            }
            NormalizedInputEvent::Cancel => {
                out_writer.write(PlayingInputEvent::Cancel);
            }
        }
    }
}

impl LaneEventSource for PlayingInputEvent {
    fn lane_input(&self) -> Option<(usize, bool)> {
        match self {
            PlayingInputEvent::PlayKeyDown(k) => Some((play_key_to_lane(*k), true)),
            PlayingInputEvent::PlayKeyUp(k) => Some((play_key_to_lane(*k), false)),
            PlayingInputEvent::ScratchDown => Some((3, true)),
            PlayingInputEvent::ScratchUp => Some((3, false)),
            PlayingInputEvent::Confirm | PlayingInputEvent::Cancel => None,
        }
    }
}

fn play_key_to_lane(key: PlayBinding) -> usize {
    match key {
        PlayBinding::Key1 => 0,
        PlayBinding::Key2 => 1,
        PlayBinding::Key3 => 2,
        PlayBinding::Key4 => 4,
        PlayBinding::Key5 => 5,
        PlayBinding::Key6 => 6,
        PlayBinding::Key7 => 7,
    }
}

pub fn update_playing_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut input_reader: MessageReader<PlayingInputEvent>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keys.just_pressed(KeyCode::KeyR) {
        next_state.set(AppState::Result);
        return;
    }

    for ev in input_reader.read() {
        if matches!(ev, PlayingInputEvent::Cancel) {
            next_state.set(AppState::Title);
            break;
        }
    }
}

pub fn cleanup_playing(mut commands: Commands, q: Query<Entity, With<GameplayEntity>>) {
    for entity in &q {
        commands.entity(entity).despawn();
    }
}
