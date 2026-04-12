use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use rand::prelude::*;
use rand::rngs::SmallRng;

use crate::component::note::GameplayEntity;
use crate::component::{
    JUDGE_LINE_Y_FROM_BOTTOM, NOTE_HEIGHT, NOTE_TRAVEL_SECONDS, lane_center_x, lane_specs,
};
use crate::resource::note::{Lane, NoteChart};

const ALL_LANES: [Lane; 8] = [
    Lane::S,
    Lane::D,
    Lane::F,
    Lane::Space,
    Lane::J,
    Lane::K,
    Lane::L,
    Lane::Shift,
];

pub fn generate_random_chart(mut chart: ResMut<NoteChart>) {
    let mut rng: SmallRng = rand::make_rng();
    let note_count: usize = rng.random_range(8..=20);
    let mut notes = Vec::with_capacity(note_count);
    let mut t = rng.random_range(0.3_f32..=0.8);
    for _ in 0..note_count {
        let lane = ALL_LANES[rng.random_range(0..ALL_LANES.len())];
        notes.push(crate::resource::note::ChartNote {
            lane,
            time_from_start_secs: t,
        });
        t += rng.random_range(0.25_f32..=0.9);
    }
    chart.notes = notes;
}

pub fn reset_score_summary(mut score_summary: ResMut<crate::resource::note::ScoreSummary>) {
    *score_summary = crate::resource::note::ScoreSummary::default();
}

#[derive(Resource, Default)]
pub struct ChartPlayback {
    pub elapsed_secs: f32,
    pub cursor: usize,
}

pub fn reset_playback(mut playback: ResMut<ChartPlayback>) {
    playback.elapsed_secs = 0.0;
    playback.cursor = 0
}

pub fn spawn_notes_from_chart(
    time: Res<Time>,
    window_q: Query<&Window, With<PrimaryWindow>>,
    mut commands: Commands,
    mut playback: ResMut<ChartPlayback>,
    chart: Res<NoteChart>,
) {
    let Ok(window) = window_q.single() else {
        return;
    };
    let half_h = window.height() * 0.5;
    let top_y = half_h + NOTE_HEIGHT * 0.5;
    let bottom_y = -half_h - NOTE_HEIGHT * 0.5;
    let judge_line_y = -half_h + JUDGE_LINE_Y_FROM_BOTTOM;
    let travel_distance = top_y - bottom_y;
    let speed = travel_distance / NOTE_TRAVEL_SECONDS;
    let time_to_judge = (top_y - judge_line_y) / speed;

    playback.elapsed_secs += time.delta_secs();
    let notes = &chart.notes;
    while playback.cursor < notes.len() {
        let note = &notes[playback.cursor];
        let spawn_time = note.time_from_start_secs - time_to_judge;
        if spawn_time <= playback.elapsed_secs {
            let lane_index = note.lane as usize;
            let lanes = lane_specs();
            let lane = lanes[lane_index];
            let x = lane_center_x(lane_index);
            commands.spawn((
                Sprite::from_color(lane.color, Vec2::new(lane.width, NOTE_HEIGHT)),
                Transform::from_xyz(x, top_y, 0.0),
                crate::component::note::Note {
                    lane_index,
                    initialized: false,
                    respawn_delay_remaining: 0.0,
                },
                GameplayEntity,
            ));
            playback.cursor += 1;
        } else {
            break;
        }
    }
}
