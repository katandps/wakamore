use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use rand::Rng;

use crate::component::note::{JudgeLine, LaneJudgeText, LaneKeyIndicator, Note};
use crate::component::{
    GR_WINDOW_MS, JUDGE_LINE_Y_FROM_BOTTOM, JUDGEMENT_DISPLAY_SECONDS,
    JUDGEMENT_TEXT_Y_FROM_BOTTOM, KEY_INDICATOR_DIAMETER, KEY_INDICATOR_Y_FROM_BOTTOM, LANE_COUNT,
    NOTE_HEIGHT, NOTE_TRAVEL_SECONDS, PG_WINDOW_MS, RESPAWN_DELAY_MAX_SECONDS,
    RESPAWN_DELAY_MIN_SECONDS, SCORE_GR, SCORE_PG, lane_center_x, lane_is_just_pressed,
    lane_is_pressed,
};
use crate::event::note::{JudgementKind, LaneInputEvent, LaneJudgementEvent};
use crate::resource::note::{LaneInputState, ScoreSummary};

pub fn reset_score_summary(mut score_summary: ResMut<ScoreSummary>) {
    *score_summary = ScoreSummary::default();
}

pub fn collect_lane_input_state(
    keys: Res<ButtonInput<KeyCode>>,
    mut input_state: ResMut<LaneInputState>,
) {
    for lane_index in 0..input_state.pressed.len() {
        input_state.pressed[lane_index] = lane_is_pressed(&keys, lane_index);
        input_state.just_pressed[lane_index] = lane_is_just_pressed(&keys, lane_index);
    }
}

pub fn emit_lane_input_events(
    input_state: Res<LaneInputState>,
    mut input_event_writer: MessageWriter<LaneInputEvent>,
) {
    for lane_index in 0..input_state.pressed.len() {
        input_event_writer.write(LaneInputEvent {
            lane_index,
            pressed: input_state.pressed[lane_index],
        });
    }
}

pub fn apply_lane_input_visuals(
    mut input_event_reader: MessageReader<LaneInputEvent>,
    mut indicator_q: Query<(&LaneKeyIndicator, &mut BackgroundColor)>,
) {
    for event in input_event_reader.read() {
        for (indicator, mut bg) in &mut indicator_q {
            if indicator.index != event.lane_index {
                continue;
            }

            if event.pressed {
                bg.0 = Color::srgb(1.0, 0.82, 0.25);
            } else {
                bg.0 = Color::srgb(0.22, 0.24, 0.28);
            }
        }
    }
}

pub fn evaluate_lane_judgement(
    window_q: Query<&Window, With<PrimaryWindow>>,
    input_state: Res<LaneInputState>,
    note_q: Query<(&Note, &Transform)>,
    mut score_summary: ResMut<ScoreSummary>,
    mut judgement_event_writer: MessageWriter<LaneJudgementEvent>,
) {
    let Ok(window) = window_q.single() else {
        return;
    };

    let half_h = window.height() * 0.5;
    let top_y = half_h + NOTE_HEIGHT * 0.5;
    let bottom_y = -half_h - NOTE_HEIGHT * 0.5;
    let judge_line_y = -half_h + JUDGE_LINE_Y_FROM_BOTTOM;
    let speed = (top_y - bottom_y) / NOTE_TRAVEL_SECONDS;

    let mut note_y_by_lane = [f32::NAN; LANE_COUNT];
    for (note, transform) in &note_q {
        if note.lane_index < note_y_by_lane.len() {
            note_y_by_lane[note.lane_index] = transform.translation.y;
        }
    }

    for lane_index in 0..input_state.just_pressed.len() {
        if !input_state.just_pressed[lane_index] {
            continue;
        }

        let note_y = note_y_by_lane[lane_index];
        if !note_y.is_finite() {
            continue;
        }

        let delta_ms = ((note_y - judge_line_y).abs() / speed) * 1000.0;
        if delta_ms <= PG_WINDOW_MS {
            score_summary.pg += 1;
            score_summary.score += SCORE_PG;
            judgement_event_writer.write(LaneJudgementEvent {
                lane_index,
                kind: JudgementKind::Pg,
            });
        } else if delta_ms <= GR_WINDOW_MS {
            score_summary.gr += 1;
            score_summary.score += SCORE_GR;
            judgement_event_writer.write(LaneJudgementEvent {
                lane_index,
                kind: JudgementKind::Gr,
            });
        } else {
            score_summary.miss += 1;
            judgement_event_writer.write(LaneJudgementEvent {
                lane_index,
                kind: JudgementKind::Miss,
            });
        }
    }
}

pub fn apply_judgement_display(
    time: Res<Time>,
    mut judgement_event_reader: MessageReader<LaneJudgementEvent>,
    mut judge_text_q: Query<(&mut LaneJudgeText, &mut Text, &mut TextColor)>,
) {
    for (mut judge_text, mut text, _) in &mut judge_text_q {
        if judge_text.remaining_secs > 0.0 {
            judge_text.remaining_secs -= time.delta_secs();
            if judge_text.remaining_secs <= 0.0 {
                text.0.clear();
            }
        }
    }

    for event in judgement_event_reader.read() {
        for (mut judge_text, mut text, mut text_color) in &mut judge_text_q {
            if judge_text.index != event.lane_index {
                continue;
            }

            match event.kind {
                JudgementKind::Pg => {
                    text.0 = "PG".to_string();
                    text_color.0 = Color::srgb(1.0, 0.92, 0.35);
                }
                JudgementKind::Gr => {
                    text.0 = "GR".to_string();
                    text_color.0 = Color::srgb(0.45, 0.95, 0.45);
                }
                JudgementKind::Miss => {
                    text.0 = "MISS".to_string();
                    text_color.0 = Color::srgb(1.0, 0.4, 0.4);
                }
            }
            judge_text.remaining_secs = JUDGEMENT_DISPLAY_SECONDS;
        }
    }
}

pub fn sync_lane_ui_layout(
    window_q: Query<&Window, With<PrimaryWindow>>,
    mut judge_line_q: Query<&mut Transform, With<JudgeLine>>,
    mut indicator_q: Query<(&LaneKeyIndicator, &mut Node), Without<LaneJudgeText>>,
    mut judge_text_q: Query<(&LaneJudgeText, &mut Node), Without<LaneKeyIndicator>>,
) {
    let Ok(window) = window_q.single() else {
        return;
    };

    let judge_line_y = -window.height() * 0.5 + JUDGE_LINE_Y_FROM_BOTTOM;
    for mut transform in &mut judge_line_q {
        transform.translation.y = judge_line_y;
    }

    for (indicator, mut node) in &mut indicator_q {
        let x = lane_center_x(indicator.index);
        let screen_x = x + window.width() * 0.5;
        node.left = Val::Px(screen_x - KEY_INDICATOR_DIAMETER * 0.5);
        node.bottom = Val::Px(KEY_INDICATOR_Y_FROM_BOTTOM - KEY_INDICATOR_DIAMETER * 0.5);
    }

    for (judge_text, mut node) in &mut judge_text_q {
        let x = lane_center_x(judge_text.index);
        let screen_x = x + window.width() * 0.5;
        node.left = Val::Px(screen_x - 16.0);
        node.bottom = Val::Px(JUDGEMENT_TEXT_Y_FROM_BOTTOM);
    }
}

pub fn animate_note(
    time: Res<Time>,
    window_q: Query<&Window, With<PrimaryWindow>>,
    mut note_q: Query<(&mut Transform, &mut Visibility, &mut Note)>,
) {
    let Ok(window) = window_q.single() else {
        return;
    };

    let half_h = window.height() * 0.5;
    let top_y = half_h + NOTE_HEIGHT * 0.5;
    let judge_line_y = -half_h + JUDGE_LINE_Y_FROM_BOTTOM;
    let bottom_y = -half_h - NOTE_HEIGHT * 0.5;
    let travel_distance = top_y - bottom_y;
    let speed = travel_distance / NOTE_TRAVEL_SECONDS;

    let mut rng = rand::thread_rng();

    for (mut transform, mut visibility, mut note) in &mut note_q {
        if !note.initialized {
            transform.translation.y = top_y;
            note.initialized = true;
        }

        if note.respawn_delay_remaining > 0.0 {
            note.respawn_delay_remaining -= time.delta_secs();
            if note.respawn_delay_remaining <= 0.0 {
                transform.translation.y = top_y;
                note.respawn_delay_remaining = 0.0;
            }
            continue;
        }

        transform.translation.y -= speed * time.delta_secs();

        if transform.translation.y < bottom_y {
            transform.translation.y = bottom_y;
            note.respawn_delay_remaining =
                rng.gen_range(RESPAWN_DELAY_MIN_SECONDS..RESPAWN_DELAY_MAX_SECONDS);
        }

        if transform.translation.y < judge_line_y {
            *visibility = Visibility::Hidden;
        } else {
            *visibility = Visibility::Visible;
        }
    }
}
