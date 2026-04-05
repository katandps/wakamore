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
const KEY_INDICATOR_DIAMETER: f32 = 18.0;
const KEY_INDICATOR_Y_FROM_BOTTOM: f32 = JUDGE_LINE_Y_FROM_BOTTOM - 30.0;
const JUDGEMENT_TEXT_Y_FROM_BOTTOM: f32 = KEY_INDICATOR_Y_FROM_BOTTOM - 24.0;
const JUDGEMENT_DISPLAY_SECONDS: f32 = 0.18;
const PG_WINDOW_MS: f32 = 20.0;
const GR_WINDOW_MS: f32 = 40.0;
const LANE_TOTAL_WIDTH: f32 =
    WHITE_NOTE_WIDTH * 4.0 + BLUE_NOTE_WIDTH * 3.0 + RED_NOTE_WIDTH + LANE_GAP * 7.0;

#[derive(Component)]
pub struct Note {
    lane_index: usize,
    initialized: bool,
    respawn_delay_remaining: f32,
}

#[derive(Component)]
pub struct LaneKeyIndicator {
    index: usize,
}

#[derive(Component)]
pub struct LaneJudgeText {
    index: usize,
    remaining_secs: f32,
}

#[derive(Component)]
pub struct JudgeLine;

struct LaneSpec {
    width: f32,
    color: Color,
}

fn lane_center_x(lane_index: usize) -> f32 {
    let lanes = lane_specs();
    let mut left = -LANE_TOTAL_WIDTH * 0.5;

    for (idx, lane) in lanes.into_iter().enumerate() {
        let center = left + lane.width * 0.5;
        if idx == lane_index {
            return center;
        }
        left += lane.width + LANE_GAP;
    }

    0.0
}

fn lane_specs() -> [LaneSpec; 8] {
    [
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
    ]
}

pub fn setup_note(commands: &mut Commands) {
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
        ));
        left += lane.width + LANE_GAP;
    }
}

fn lane_is_pressed(keys: &ButtonInput<KeyCode>, lane_index: usize) -> bool {
    match lane_index {
        0 => keys.pressed(KeyCode::KeyS),
        1 => keys.pressed(KeyCode::KeyD),
        2 => keys.pressed(KeyCode::KeyF),
        3 => keys.pressed(KeyCode::Space),
        4 => keys.pressed(KeyCode::KeyJ),
        5 => keys.pressed(KeyCode::KeyK),
        6 => keys.pressed(KeyCode::KeyL),
        7 => keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight),
        _ => false,
    }
}

fn lane_is_just_pressed(keys: &ButtonInput<KeyCode>, lane_index: usize) -> bool {
    match lane_index {
        0 => keys.just_pressed(KeyCode::KeyS),
        1 => keys.just_pressed(KeyCode::KeyD),
        2 => keys.just_pressed(KeyCode::KeyF),
        3 => keys.just_pressed(KeyCode::Space),
        4 => keys.just_pressed(KeyCode::KeyJ),
        5 => keys.just_pressed(KeyCode::KeyK),
        6 => keys.just_pressed(KeyCode::KeyL),
        7 => keys.just_pressed(KeyCode::ShiftLeft) || keys.just_pressed(KeyCode::ShiftRight),
        _ => false,
    }
}

pub fn handle_lane_input(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    window_q: Query<&Window, With<PrimaryWindow>>,
    mut indicator_q: Query<(&LaneKeyIndicator, &mut BackgroundColor)>,
    note_q: Query<(&Note, &Transform)>,
    mut judge_text_q: Query<(&mut LaneJudgeText, &mut Text, &mut TextColor)>,
) {
    let Ok(window) = window_q.single() else {
        return;
    };

    let half_h = window.height() * 0.5;
    let top_y = half_h + NOTE_HEIGHT * 0.5;
    let bottom_y = -half_h - NOTE_HEIGHT * 0.5;
    let judge_line_y = -half_h + JUDGE_LINE_Y_FROM_BOTTOM;
    let speed = (top_y - bottom_y) / NOTE_TRAVEL_SECONDS;

    let mut note_y_by_lane = [f32::NAN; 8];
    for (note, transform) in &note_q {
        if note.lane_index < note_y_by_lane.len() {
            note_y_by_lane[note.lane_index] = transform.translation.y;
        }
    }

    for (indicator, mut bg) in &mut indicator_q {
        if lane_is_pressed(&keys, indicator.index) {
            bg.0 = Color::srgb(1.0, 0.82, 0.25);
        } else {
            bg.0 = Color::srgb(0.22, 0.24, 0.28);
        }
    }

    for (mut judge_text, mut text, mut text_color) in &mut judge_text_q {
        if judge_text.remaining_secs > 0.0 {
            judge_text.remaining_secs -= time.delta_secs();
            if judge_text.remaining_secs <= 0.0 {
                text.0.clear();
            }
        }

        if !lane_is_just_pressed(&keys, judge_text.index) {
            continue;
        }

        let note_y = note_y_by_lane[judge_text.index];
        if !note_y.is_finite() {
            continue;
        }

        let delta_ms = ((note_y - judge_line_y).abs() / speed) * 1000.0;

        if delta_ms <= PG_WINDOW_MS {
            text.0 = "PG".to_string();
            text_color.0 = Color::srgb(1.0, 0.92, 0.35);
            judge_text.remaining_secs = JUDGEMENT_DISPLAY_SECONDS;
        } else if delta_ms <= GR_WINDOW_MS {
            text.0 = "GR".to_string();
            text_color.0 = Color::srgb(0.45, 0.95, 0.45);
            judge_text.remaining_secs = JUDGEMENT_DISPLAY_SECONDS;
        }
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
        ));

        left += lane.width + LANE_GAP;
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
