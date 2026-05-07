use bevy::prelude::*;
use common::ResultInputEvent;

use crate::component::ScoreSummary;
use crate::state::{AppState, ResultEntity};
use wakamore_render::ResultSummaryView;

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
            ResultInputEvent::Cancel => {
                next_state.set(AppState::Playing);
                break;
            }
        }
    }
}

pub fn cleanup_result(mut commands: Commands, q: Query<Entity, With<ResultEntity>>) {
    for entity in &q {
        commands.entity(entity).despawn();
    }
}
