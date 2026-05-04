use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use common::{JudgementKind, LaneInputEvent, LaneJudgementEvent};
use std::marker::PhantomData;

pub const LANE_COUNT: usize = 8;
pub const WHITE_NOTE_WIDTH: f32 = 60.0;
pub const BLUE_NOTE_WIDTH: f32 = 48.0;
pub const RED_NOTE_WIDTH: f32 = 108.0;
pub const NOTE_HEIGHT: f32 = 24.0;
pub const LANE_GAP: f32 = 3.0;
pub const NOTE_TRAVEL_SECONDS: f32 = 0.5;
pub const JUDGE_LINE_Y_FROM_BOTTOM: f32 = 200.0;
pub const JUDGE_LINE_THICKNESS: f32 = 4.0;
pub const KEY_INDICATOR_DIAMETER: f32 = 18.0;
pub const KEY_INDICATOR_Y_FROM_BOTTOM: f32 = JUDGE_LINE_Y_FROM_BOTTOM - 30.0;
pub const JUDGEMENT_TEXT_Y_FROM_BOTTOM: f32 = KEY_INDICATOR_Y_FROM_BOTTOM - 24.0;
pub const JUDGEMENT_DISPLAY_SECONDS: f32 = 0.18;
pub const LANE_TOTAL_WIDTH: f32 =
    WHITE_NOTE_WIDTH * 4.0 + BLUE_NOTE_WIDTH * 3.0 + RED_NOTE_WIDTH + LANE_GAP * 7.0;

#[derive(Clone, Copy)]
pub struct LaneSpec {
    pub width: f32,
    pub color: Color,
}

#[derive(Component)]
pub struct GameplayEntity;

#[derive(Component)]
pub struct LaneKeyIndicator {
    pub index: usize,
}

#[derive(Component)]
pub struct LaneJudgeText {
    pub index: usize,
    pub remaining_secs: f32,
}

#[derive(Component)]
pub struct JudgeLine;

#[derive(Resource)]
pub struct UiFont(pub Handle<Font>);

#[derive(Component)]
pub struct TitleEntity;

#[derive(Component)]
pub struct ResultEntity;

pub trait ResultSummaryView {
    fn score(&self) -> u32;
    fn pg(&self) -> u32;
    fn gr(&self) -> u32;
    fn miss(&self) -> u32;
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum RenderUpdateSet {
    InputVisuals,
    JudgementDisplay,
    LaneLayout,
}

pub struct WakamoreRenderPlugin<S: States + Clone, T: Resource + ResultSummaryView> {
    title_state: S,
    playing_state: S,
    result_state: S,
    _summary: PhantomData<T>,
}

impl<S: States + Clone, T: Resource + ResultSummaryView> WakamoreRenderPlugin<S, T> {
    pub fn new(title_state: S, playing_state: S, result_state: S) -> Self {
        Self {
            title_state,
            playing_state,
            result_state,
            _summary: PhantomData,
        }
    }
}

impl<S: States + Clone, T: Resource + ResultSummaryView> Plugin for WakamoreRenderPlugin<S, T> {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_camera, setup_ui_font))
            .add_systems(OnEnter(self.title_state.clone()), setup_title)
            .add_systems(OnEnter(self.playing_state.clone()), setup_judge_line)
            .add_systems(
                Update,
                (
                    apply_lane_input_visuals.in_set(RenderUpdateSet::InputVisuals),
                    apply_judgement_display.in_set(RenderUpdateSet::JudgementDisplay),
                    sync_lane_ui_layout.in_set(RenderUpdateSet::LaneLayout),
                )
                    .run_if(in_state(self.playing_state.clone())),
            )
            .add_systems(OnEnter(self.result_state.clone()), setup_result::<T>);
    }
}

pub fn lane_specs() -> [LaneSpec; LANE_COUNT] {
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

pub fn lane_center_x(lane_index: usize) -> f32 {
    let mut left = -LANE_TOTAL_WIDTH * 0.5;

    for (idx, lane) in lane_specs().into_iter().enumerate() {
        let center = left + lane.width * 0.5;
        if idx == lane_index {
            return center;
        }
        left += lane.width + LANE_GAP;
    }

    0.0
}

pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d::default());
}

pub fn setup_ui_font(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/NotoSansJP-Regular.ttf");
    commands.insert_resource(UiFont(font));
}

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

pub fn setup_result<T: Resource + ResultSummaryView>(
    mut commands: Commands,
    score_summary: Res<T>,
) {
    commands.spawn((
        Text::new(format!(
            "Result\nScore: {}\nPG: {}  GR: {}  MISS: {}\nPress Enter: Title\nPress Space: Retry",
            score_summary.score(),
            score_summary.pg(),
            score_summary.gr(),
            score_summary.miss()
        )),
        TextFont {
            font_size: 42.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(33.0),
            top: Val::Percent(40.0),
            ..default()
        },
        ResultEntity,
    ));
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
