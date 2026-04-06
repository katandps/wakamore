use bevy::prelude::*;

use crate::state::{AppState, TitleEntity};

pub fn setup_title(mut commands: Commands) {
    commands.spawn((
        Text::new("Press Enter to Start"),
        TextFont {
            font_size: 46.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(36.0),
            top: Val::Percent(45.0),
            ..default()
        },
        TitleEntity,
    ));
}

pub fn update_title_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keys.just_pressed(KeyCode::Enter) {
        next_state.set(AppState::Playing);
    }
}

pub fn cleanup_title(mut commands: Commands, q: Query<Entity, With<TitleEntity>>) {
    for entity in &q {
        commands.entity(entity).despawn();
    }
}
