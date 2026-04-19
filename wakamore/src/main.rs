mod component;
mod entity;
mod resource;
mod state;
mod system;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy::winit::WinitSettings;
use component::fps::{FpsHistory, setup_fps, update_fps_text};
use emitter::{
    emit_gamepad_button_lane_input, input_events_to_lane_events, record_lane_raw_events,
};
use entity::note::{setup_judge_line, setup_note};
use input::poll_key_events;
use resource::note::{LaneInputState, NoteChart, ScoreSummary};
use state::{
    AppState, cleanup_playing, cleanup_result, cleanup_title, setup_result, setup_title,
    update_playing_input, update_result_input, update_title_input,
};
use system::input_log::record_judgement_to_log;
use system::note_animate::animate_note;
use system::note_input::apply_lane_input_visuals;
use system::note_judge::{apply_judgement_display, evaluate_lane_judgement};
use system::note_spawn::{
    check_playback_finished, prepare_chart, reset_playback, reset_score_summary,
    spawn_notes_from_chart,
};
use system::note_ui::sync_lane_ui_layout;

fn main() {
    // try to load bindings from `bindings.toml` in the current working directory;
    // fall back to defaults if loading/parsing fails.
    let initial_bindings = match input::Bindings::from_file("bindings.toml") {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Failed to load bindings.toml: {}. Using defaults.", e);
            input::Bindings::with_defaults()
        }
    };

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
        .insert_resource(common::KeyToActionResource(Box::new(initial_bindings)))
        .init_resource::<common::InputLog>()
        .init_resource::<common::LastRawByLane>()
        .init_resource::<component::ChartPlayback>()
        .insert_resource(NoteChart::demo())
        .init_resource::<ScoreSummary>()
        .add_message::<common::LaneInputEvent>()
        .add_message::<common::InputEvent>()
        .add_message::<common::LaneJudgementEvent>()
        .add_systems(Startup, (setup_camera, setup_fps))
        .add_systems(Startup, setup_ui_font)
        .add_systems(OnEnter(AppState::Title), setup_title)
        .add_systems(Update, update_title_input.run_if(in_state(AppState::Title)))
        .add_systems(OnExit(AppState::Title), cleanup_title)
        .add_systems(
            OnEnter(AppState::Playing),
            (
                prepare_chart,
                reset_score_summary,
                reset_playback,
                setup_note,
                setup_judge_line,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                poll_key_events,
                emit_gamepad_button_lane_input,
                input_events_to_lane_events,
                record_lane_raw_events,
                apply_lane_input_visuals,
                evaluate_lane_judgement,
                record_judgement_to_log,
                apply_judgement_display,
                animate_note,
                spawn_notes_from_chart,
                check_playback_finished,
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

#[derive(Resource)]
pub struct UiFont(pub Handle<Font>);

fn setup_ui_font(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/NotoSansJP-Regular.ttf");
    commands.insert_resource(UiFont(font));
}
// State-specific systems are provided by `state` module submodules.
