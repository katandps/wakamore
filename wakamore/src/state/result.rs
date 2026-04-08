use bevy::prelude::*;

use crate::component::ScoreSummary;
use crate::state::{AppState, ResultEntity};

pub fn setup_result(mut commands: Commands, score_summary: Res<ScoreSummary>) {
    commands.spawn((
        Text::new(format!(
            "Result\nScore: {}\nPG: {}  GR: {}  MISS: {}\nPress Enter: Title\nPress Space: Retry",
            score_summary.score, score_summary.pg, score_summary.gr, score_summary.miss
        )),
        TextFont {
            font_size: 42.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(33.0),
            top: Val::Percent(40.0),
            ..default()
        },
        ResultEntity,
    ));
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
