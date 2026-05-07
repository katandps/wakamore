use bevy::prelude::*;
use common::PlayingInputEvent;

use crate::component::GameplayEntity;
use crate::state::AppState;

pub fn update_playing_input(
    mut input_reader: MessageReader<PlayingInputEvent>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for ev in input_reader.read() {
        if matches!(ev, PlayingInputEvent::Abort) {
            next_state.set(AppState::Result);
            break;
        }
    }
}

pub fn cleanup_playing(mut commands: Commands, q: Query<Entity, With<GameplayEntity>>) {
    for entity in &q {
        commands.entity(entity).despawn();
    }
}
