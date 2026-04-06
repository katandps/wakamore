use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::component::note::LaneJudgeText;
use crate::component::{JUDGE_LINE_Y_FROM_BOTTOM, NOTE_HEIGHT, NOTE_TRAVEL_SECONDS, PG_WINDOW_MS, GR_WINDOW_MS, SCORE_PG, SCORE_GR, LANE_COUNT};
use crate::resource::note::LaneInputState;
use crate::event::note::{LaneJudgementEvent, JudgementKind};
use crate::resource::note::ScoreSummary;

pub fn evaluate_lane_judgement(
    window_q: Query<&Window, With<PrimaryWindow>>,
    input_state: Res<LaneInputState>,
    note_q: Query<(&crate::component::note::Note, &Transform)>,
    mut score_summary: ResMut<ScoreSummary>,
    mut judgement_event_writer: MessageWriter<LaneJudgementEvent>,
) {
    let Ok(window) = window_q.single() else { return; };
    let half_h = window.height() * 0.5;
    let top_y = half_h + NOTE_HEIGHT * 0.5;
    let bottom_y = -half_h - NOTE_HEIGHT * 0.5;
    let judge_line_y = -half_h + JUDGE_LINE_Y_FROM_BOTTOM;
    let speed = (top_y - bottom_y) / NOTE_TRAVEL_SECONDS;

    let mut note_y_by_lane = [f32::NAN; LANE_COUNT];
    for (note, transform) in &note_q { if note.lane_index < note_y_by_lane.len() { note_y_by_lane[note.lane_index] = transform.translation.y; } }

    for lane_index in 0..input_state.just_pressed.len() {
        if !input_state.just_pressed[lane_index] { continue; }
        let note_y = note_y_by_lane[lane_index]; if !note_y.is_finite() { continue; }
        let delta_ms = ((note_y - judge_line_y).abs() / speed) * 1000.0;
        if delta_ms <= PG_WINDOW_MS {
            score_summary.pg += 1; score_summary.score += SCORE_PG;
            judgement_event_writer.write(LaneJudgementEvent { lane_index, kind: JudgementKind::Pg });
        } else if delta_ms <= GR_WINDOW_MS {
            score_summary.gr += 1; score_summary.score += SCORE_GR;
            judgement_event_writer.write(LaneJudgementEvent { lane_index, kind: JudgementKind::Gr });
        } else {
            score_summary.miss += 1;
            judgement_event_writer.write(LaneJudgementEvent { lane_index, kind: JudgementKind::Miss });
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
            if judge_text.remaining_secs <= 0.0 { text.0.clear(); }
        }
    }
    for event in judgement_event_reader.read() {
        for (mut judge_text, mut text, mut text_color) in &mut judge_text_q {
            if judge_text.index != event.lane_index { continue; }
            match event.kind {
                JudgementKind::Pg => { text.0 = "PG".to_string(); text_color.0 = Color::srgb(1.0, 0.92, 0.35); }
                JudgementKind::Gr => { text.0 = "GR".to_string(); text_color.0 = Color::srgb(0.45, 0.95, 0.45); }
                JudgementKind::Miss => { text.0 = "MISS".to_string(); text_color.0 = Color::srgb(1.0, 0.4, 0.4); }
            }
            judge_text.remaining_secs = crate::component::JUDGEMENT_DISPLAY_SECONDS;
        }
    }
}
