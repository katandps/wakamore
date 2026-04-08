use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use rand::Rng;

use crate::component::note::Note;
use crate::component::{JUDGE_LINE_Y_FROM_BOTTOM, NOTE_HEIGHT, NOTE_TRAVEL_SECONDS, RESPAWN_DELAY_MAX_SECONDS, RESPAWN_DELAY_MIN_SECONDS};

pub fn animate_note(
    time: Res<Time>,
    window_q: Query<&Window, With<PrimaryWindow>>,
    mut note_q: Query<(&mut Transform, &mut Visibility, &mut Note)>,
) {
    let Ok(window) = window_q.single() else { return; };
    let half_h = window.height() * 0.5;
    let top_y = half_h + NOTE_HEIGHT * 0.5;
    let judge_line_y = -half_h + JUDGE_LINE_Y_FROM_BOTTOM;
    let bottom_y = -half_h - NOTE_HEIGHT * 0.5;
    let travel_distance = top_y - bottom_y;
    let speed = travel_distance / NOTE_TRAVEL_SECONDS;
    let mut rng = rand::thread_rng();
    for (mut transform, mut visibility, mut note) in &mut note_q {
        if !note.initialized { transform.translation.y = top_y; note.initialized = true; }
        if note.respawn_delay_remaining > 0.0 {
            note.respawn_delay_remaining -= time.delta_secs();
            if note.respawn_delay_remaining <= 0.0 { transform.translation.y = top_y; note.respawn_delay_remaining = 0.0; }
            continue;
        }
        transform.translation.y -= speed * time.delta_secs();
        if transform.translation.y < bottom_y {
            transform.translation.y = bottom_y;
            note.respawn_delay_remaining = rng.gen_range(RESPAWN_DELAY_MIN_SECONDS..RESPAWN_DELAY_MAX_SECONDS);
        }
        if transform.translation.y < judge_line_y { *visibility = Visibility::Hidden; } else { *visibility = Visibility::Visible; }
    }
}
