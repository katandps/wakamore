use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::component::note::{GameplayEntity, JudgeLine, LaneJudgeText, LaneKeyIndicator, Note};
use crate::component::{
    JUDGE_LINE_THICKNESS, JUDGE_LINE_Y_FROM_BOTTOM, JUDGEMENT_TEXT_Y_FROM_BOTTOM,
    KEY_INDICATOR_DIAMETER, KEY_INDICATOR_Y_FROM_BOTTOM, LANE_GAP, LANE_TOTAL_WIDTH, NOTE_HEIGHT,
    lane_specs,
};

pub fn setup_note(mut commands: Commands) {
    let lanes = lane_specs();

    let total_width = lanes.iter().map(|lane| lane.width).sum::<f32>()
        + LANE_GAP * (lanes.len().saturating_sub(1) as f32);
    let mut left = -total_width * 0.5;

    for (lane_index, lane) in lanes.into_iter().enumerate() {
        let x = left + lane.width * 0.5;
        commands.spawn((
            Sprite::from_color(lane.color, Vec2::new(lane.width, NOTE_HEIGHT)),
            Transform::from_xyz(x, 0.0, 0.0),
            Note {
                lane_index,
                initialized: false,
                respawn_delay_remaining: 0.0,
            },
            GameplayEntity,
        ));
        left += lane.width + LANE_GAP;
    }
}

pub fn setup_judge_line(mut commands: Commands, window_q: Query<&Window, With<PrimaryWindow>>) {
    let Ok(window) = window_q.single() else {
        return;
    };

    let lanes = lane_specs();
    let mut left = -LANE_TOTAL_WIDTH * 0.5;

    let y = -window.height() * 0.5 + JUDGE_LINE_Y_FROM_BOTTOM;
    commands.spawn((
        Sprite::from_color(
            Color::srgb(1.0, 0.95, 0.2),
            Vec2::new(LANE_TOTAL_WIDTH, JUDGE_LINE_THICKNESS),
        ),
        Transform::from_xyz(0.0, y, 1.0),
        JudgeLine,
        GameplayEntity,
    ));

    for (lane_index, lane) in lanes.into_iter().enumerate() {
        let x = left + lane.width * 0.5;
        let screen_x = x + window.width() * 0.5;
        commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Px(KEY_INDICATOR_DIAMETER),
                height: Val::Px(KEY_INDICATOR_DIAMETER),
                left: Val::Px(screen_x - KEY_INDICATOR_DIAMETER * 0.5),
                bottom: Val::Px(KEY_INDICATOR_Y_FROM_BOTTOM - KEY_INDICATOR_DIAMETER * 0.5),
                border_radius: BorderRadius::MAX,
                ..default()
            },
            BackgroundColor(Color::srgb(0.22, 0.24, 0.28)),
            LaneKeyIndicator { index: lane_index },
            GameplayEntity,
        ));

        commands.spawn((
            Text::new(""),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Node {
                position_type: PositionType::Absolute,
                width: Val::Px(32.0),
                left: Val::Px(screen_x - 16.0),
                bottom: Val::Px(JUDGEMENT_TEXT_Y_FROM_BOTTOM),
                ..default()
            },
            LaneJudgeText {
                index: lane_index,
                remaining_secs: 0.0,
            },
            GameplayEntity,
        ));

        left += lane.width + LANE_GAP;
    }
}
