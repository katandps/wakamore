use bevy::prelude::*;
use common::TitleInputEvent;

use crate::state::{AppState, TitleEntity};

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
