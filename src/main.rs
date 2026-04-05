mod component;
mod entity;
mod event;
mod resource;
mod system;

use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy::winit::WinitSettings;
use component::{
    FpsHistory, GameplayEntity, LaneInputEvent, LaneInputState, LaneJudgementEvent, ScoreSummary,
    animate_note, apply_judgement_display, apply_lane_input_visuals, collect_lane_input_state,
    emit_lane_input_events, evaluate_lane_judgement, reset_score_summary, setup_fps,
    setup_judge_line, setup_note, sync_lane_ui_layout, update_fps_text,
};

#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
enum AppState {
    #[default]
    Title,
    Playing,
    Result,
}

#[derive(Component)]
struct TitleEntity;

#[derive(Component)]
struct ResultEntity;

fn main() {
    App::new()
        .insert_resource(WinitSettings::game())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                present_mode: PresentMode::AutoNoVsync,
                resolution: (1920, 1080).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .init_state::<AppState>()
        .init_resource::<FpsHistory>()
        .init_resource::<LaneInputState>()
        .init_resource::<ScoreSummary>()
        .add_message::<LaneInputEvent>()
        .add_message::<LaneJudgementEvent>()
        .add_systems(Startup, (setup_camera, setup_fps))
        .add_systems(OnEnter(AppState::Title), setup_title)
        .add_systems(Update, update_title_input.run_if(in_state(AppState::Title)))
        .add_systems(OnExit(AppState::Title), cleanup_title)
        .add_systems(
            OnEnter(AppState::Playing),
            (reset_score_summary, setup_note, setup_judge_line),
        )
        .add_systems(
            Update,
            (
                collect_lane_input_state,
                emit_lane_input_events,
                apply_lane_input_visuals,
                evaluate_lane_judgement,
                apply_judgement_display,
                animate_note,
                sync_lane_ui_layout,
            )
                .chain()
                .run_if(in_state(AppState::Playing)),
        )
        .add_systems(Update, update_fps_text)
        .add_systems(
            Update,
            update_playing_input.run_if(in_state(AppState::Playing)),
        )
        .add_systems(OnExit(AppState::Playing), cleanup_playing)
        .add_systems(OnEnter(AppState::Result), setup_result)
        .add_systems(
            Update,
            update_result_input.run_if(in_state(AppState::Result)),
        )
        .add_systems(OnExit(AppState::Result), cleanup_result)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d::default());
}

fn setup_title(mut commands: Commands) {
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

fn update_title_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keys.just_pressed(KeyCode::Enter) {
        next_state.set(AppState::Playing);
    }
}

fn update_playing_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keys.just_pressed(KeyCode::KeyR) {
        next_state.set(AppState::Result);
        return;
    }

    if keys.just_pressed(KeyCode::Escape) {
        next_state.set(AppState::Title);
    }
}

fn setup_result(mut commands: Commands, score_summary: Res<ScoreSummary>) {
    commands.spawn((
        Text::new(format!(
            "Result\nScore: {}\nPG: {}  GR: {}  MISS: {}\nPress Enter: Title\nPress Space: Retry",
            score_summary.score, score_summary.pg, score_summary.gr, score_summary.miss
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

fn update_result_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keys.just_pressed(KeyCode::Enter) {
        next_state.set(AppState::Title);
        return;
    }

    if keys.just_pressed(KeyCode::Space) {
        next_state.set(AppState::Playing);
    }
}

fn cleanup_title(mut commands: Commands, q: Query<Entity, With<TitleEntity>>) {
    for entity in &q {
        commands.entity(entity).despawn();
    }
}

fn cleanup_playing(mut commands: Commands, q: Query<Entity, With<GameplayEntity>>) {
    for entity in &q {
        commands.entity(entity).despawn();
    }
}

fn cleanup_result(mut commands: Commands, q: Query<Entity, With<ResultEntity>>) {
    for entity in &q {
        commands.entity(entity).despawn();
    }
}
