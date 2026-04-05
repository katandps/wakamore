use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use rand::Rng;

const WHITE_NOTE_WIDTH: f32 = 60.0;
const BLUE_NOTE_WIDTH: f32 = 48.0;
const RED_NOTE_WIDTH: f32 = 108.0;
const NOTE_HEIGHT: f32 = 24.0;
const LANE_GAP: f32 = 3.0;
const NOTE_TRAVEL_SECONDS: f32 = 0.5;
const RESPAWN_DELAY_MIN_SECONDS: f32 = 0.08;
const RESPAWN_DELAY_MAX_SECONDS: f32 = 0.25;
const JUDGE_LINE_Y_FROM_BOTTOM: f32 = 200.0;
const JUDGE_LINE_THICKNESS: f32 = 4.0;
const LANE_TOTAL_WIDTH: f32 =
    WHITE_NOTE_WIDTH * 4.0 + BLUE_NOTE_WIDTH * 3.0 + RED_NOTE_WIDTH + LANE_GAP * 7.0;

#[derive(Component)]
pub struct Note {
    initialized: bool,
    respawn_delay_remaining: f32,
}

#[derive(Component)]
pub struct JudgeLine;

struct LaneSpec {
    width: f32,
    color: Color,
}

pub fn setup_note(commands: &mut Commands) {
    let lanes = [
        LaneSpec {
            width: WHITE_NOTE_WIDTH,
            color: Color::WHITE,
        },
        LaneSpec {
            width: BLUE_NOTE_WIDTH,
            color: Color::srgb(0.0, 0.5, 1.0),
        },
        LaneSpec {
            width: WHITE_NOTE_WIDTH,
            color: Color::WHITE,
        },
        LaneSpec {
            width: BLUE_NOTE_WIDTH,
            color: Color::srgb(0.0, 0.5, 1.0),
        },
        LaneSpec {
            width: WHITE_NOTE_WIDTH,
            color: Color::WHITE,
        },
        LaneSpec {
            width: BLUE_NOTE_WIDTH,
            color: Color::srgb(0.0, 0.5, 1.0),
        },
        LaneSpec {
            width: WHITE_NOTE_WIDTH,
            color: Color::WHITE,
        },
        LaneSpec {
            width: RED_NOTE_WIDTH,
            color: Color::srgb(1.0, 0.0, 0.0),
        },
    ];

    let total_width = lanes.iter().map(|lane| lane.width).sum::<f32>()
        + LANE_GAP * (lanes.len().saturating_sub(1) as f32);
    let mut left = -total_width * 0.5;

    for lane in lanes {
        let x = left + lane.width * 0.5;
        commands.spawn((
            Sprite::from_color(lane.color, Vec2::new(lane.width, NOTE_HEIGHT)),
            Transform::from_xyz(x, 0.0, 0.0),
            Note {
                initialized: false,
                respawn_delay_remaining: 0.0,
            },
        ));
        left += lane.width + LANE_GAP;
    }
}

pub fn setup_judge_line(mut commands: Commands, window_q: Query<&Window, With<PrimaryWindow>>) {
    let Ok(window) = window_q.single() else {
        return;
    };

    let y = -window.height() * 0.5 + JUDGE_LINE_Y_FROM_BOTTOM;
    commands.spawn((
        Sprite::from_color(
            Color::srgb(1.0, 0.95, 0.2),
            Vec2::new(LANE_TOTAL_WIDTH, JUDGE_LINE_THICKNESS),
        ),
        Transform::from_xyz(0.0, y, 1.0),
        JudgeLine,
    ));
}

pub fn animate_note(
    time: Res<Time>,
    window_q: Query<&Window, With<PrimaryWindow>>,
    mut note_q: Query<(&mut Transform, &mut Note)>,
) {
    let Ok(window) = window_q.single() else {
        return;
    };

    let half_h = window.height() * 0.5;
    let top_y = half_h + NOTE_HEIGHT * 0.5;
    let bottom_y = -half_h - NOTE_HEIGHT * 0.5;
    let travel_distance = top_y - bottom_y;
    let speed = travel_distance / NOTE_TRAVEL_SECONDS;

    let mut rng = rand::thread_rng();

    for (mut transform, mut note) in &mut note_q {
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
    }
}
