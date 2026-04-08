use bevy::prelude::*;

use crate::component::GameplayEntity;
use crate::state::AppState;

pub fn update_playing_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keys.just_pressed(KeyCode::KeyR) {
        next_state.set(AppState::Result);
        return;
    }

    if keys.just_pressed(KeyCode::Escape) {
        next_state.set(AppState::Title);
    }
}

pub fn cleanup_playing(mut commands: Commands, q: Query<Entity, With<GameplayEntity>>) {
    for entity in &q {
        commands.entity(entity).despawn();
    }
}
