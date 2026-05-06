use bevy::prelude::*;
use common::InputEvent;
use input::NormalizedInputEvent;

use crate::component::ScoreSummary;
use crate::state::{AppState, ResultEntity};
use wakamore_render::ResultSummaryView;

#[derive(Event, Message, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResultInputEvent {
    ScratchDown,
    ScratchUp,
    Confirm,
    Cancel,
}

impl InputEvent for ResultInputEvent {
}

pub fn map_result_input_events(
    mut in_reader: MessageReader<NormalizedInputEvent>,
    mut out_writer: MessageWriter<ResultInputEvent>,
) {
    for ev in in_reader.read() {
        match ev {
            NormalizedInputEvent::ScratchDown => {
                out_writer.write(ResultInputEvent::ScratchDown);
            }
            NormalizedInputEvent::ScratchUp => {
                out_writer.write(ResultInputEvent::ScratchUp);
            }
            NormalizedInputEvent::Confirm => {
                out_writer.write(ResultInputEvent::Confirm);
            }
            NormalizedInputEvent::Cancel => {
                out_writer.write(ResultInputEvent::Cancel);
            }
            _ => {}
        }
    }
}

impl ResultSummaryView for ScoreSummary {
    fn score(&self) -> u32 {
        self.score
    }

    fn pg(&self) -> u32 {
        self.pg
    }

    fn gr(&self) -> u32 {
        self.gr
    }

    fn miss(&self) -> u32 {
        self.miss
    }
}

pub fn update_result_input(
    mut input_reader: MessageReader<ResultInputEvent>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for ev in input_reader.read() {
        match ev {
            ResultInputEvent::Confirm => {
                next_state.set(AppState::Title);
                break;
            }
            ResultInputEvent::ScratchDown => {
                next_state.set(AppState::Playing);
                break;
            }
            _ => {}
        }
    }
}

pub fn cleanup_result(mut commands: Commands, q: Query<Entity, With<ResultEntity>>) {
    for entity in &q {
        commands.entity(entity).despawn();
    }
}
