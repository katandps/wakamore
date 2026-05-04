use bevy::prelude::*;

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
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keys.just_pressed(KeyCode::Enter) {
        next_state.set(AppState::Title);
        return;
    }

    if keys.just_pressed(KeyCode::Space) {
        next_state.set(AppState::Playing);
    }
}

pub fn cleanup_result(mut commands: Commands, q: Query<Entity, With<ResultEntity>>) {
    for entity in &q {
        commands.entity(entity).despawn();
    }
}
