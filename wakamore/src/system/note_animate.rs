use bevy::prelude::*;
use bevy::window::PrimaryWindow;
// respawn removed; no RNG required

use crate::component::note::Note;
use crate::component::{JUDGE_LINE_Y_FROM_BOTTOM, NOTE_HEIGHT, NOTE_TRAVEL_SECONDS};
use crate::system::note_spawn::ChartPlayback;

pub fn animate_note(
    window_q: Query<&Window, With<PrimaryWindow>>,
    mut commands: Commands,
    mut note_q: Query<(Entity, &mut Transform, &mut Visibility, &Note)>,
    playback: Res<ChartPlayback>,
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
    for (entity, mut transform, mut visibility, note) in &mut note_q {
        // Compute exact position from scheduled time and playback elapsed time:
        // y = judge_line_y + (scheduled_time - elapsed) * speed
        let time_until_judge = note.scheduled_time_secs - playback.elapsed_secs;
        let mut y = judge_line_y + time_until_judge * speed;

        // Clamp to top; if below bottom, despawn the entity (no respawn)
        if y > top_y {
            y = top_y;
        }
        if y < bottom_y {
            commands.entity(entity).despawn();
            continue;
        }

        transform.translation.y = y;

        // Visibility based on judge line
        if transform.translation.y < judge_line_y {
            *visibility = Visibility::Hidden;
        } else {
            *visibility = Visibility::Visible;
        }
    }
}
