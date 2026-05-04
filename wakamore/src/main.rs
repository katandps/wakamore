mod component;
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
use input::poll_key_events;
use resource::note::{NoteChart, ScoreSummary};
use state::{
    AppState, cleanup_playing, cleanup_result, cleanup_title, update_playing_input,
    update_result_input, update_title_input,
};
use system::input_log::record_judgement_to_log;
use system::note_animate::animate_note;
use system::note_judge::evaluate_lane_judgement;
use system::note_spawn::{
    check_playback_finished, prepare_chart, reset_playback, reset_score_summary,
    spawn_notes_from_chart,
};
use wakamore_render::{RenderUpdateSet, WakamoreRenderPlugin};

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
enum GameplayUpdateSet {
    LaneEvents,
    Judgement,
    JudgementLog,
    Animation,
    Spawn,
}

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
        .add_plugins(WakamoreRenderPlugin::<AppState, ScoreSummary>::new(
            AppState::Title,
            AppState::Playing,
            AppState::Result,
        ))
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .init_state::<AppState>()
        .init_resource::<FpsHistory>()
        .insert_resource(common::KeyToActionResource(Box::new(initial_bindings)))
        .init_resource::<common::InputLog>()
        .init_resource::<common::LastRawByLane>()
        .init_resource::<component::ChartPlayback>()
        .insert_resource(NoteChart::demo())
        .init_resource::<ScoreSummary>()
        .add_message::<common::LaneInputEvent>()
        .add_message::<common::InputEvent>()
        .add_message::<common::LaneJudgementEvent>()
        .configure_sets(
            Update,
            (
                GameplayUpdateSet::LaneEvents,
                RenderUpdateSet::InputVisuals,
                GameplayUpdateSet::Judgement,
                GameplayUpdateSet::JudgementLog,
                RenderUpdateSet::JudgementDisplay,
                GameplayUpdateSet::Animation,
                GameplayUpdateSet::Spawn,
                RenderUpdateSet::LaneLayout,
            )
                .chain(),
        )
        .add_systems(Startup, setup_fps)
        .add_systems(Update, update_title_input.run_if(in_state(AppState::Title)))
        .add_systems(OnExit(AppState::Title), cleanup_title)
        .add_systems(
            OnEnter(AppState::Playing),
            (prepare_chart, reset_score_summary, reset_playback).chain(),
        )
        .add_systems(
            Update,
            (
                poll_key_events,
                emit_gamepad_button_lane_input,
                input_events_to_lane_events,
                record_lane_raw_events,
            )
                .chain()
                .in_set(GameplayUpdateSet::LaneEvents)
                .run_if(in_state(AppState::Playing)),
        )
        .add_systems(
            Update,
            evaluate_lane_judgement
                .in_set(GameplayUpdateSet::Judgement)
                .run_if(in_state(AppState::Playing)),
        )
        .add_systems(
            Update,
            record_judgement_to_log
                .in_set(GameplayUpdateSet::JudgementLog)
                .run_if(in_state(AppState::Playing)),
        )
        .add_systems(
            Update,
            animate_note
                .in_set(GameplayUpdateSet::Animation)
                .run_if(in_state(AppState::Playing)),
        )
        .add_systems(
            Update,
            (spawn_notes_from_chart, check_playback_finished)
                .chain()
                .in_set(GameplayUpdateSet::Spawn)
                .run_if(in_state(AppState::Playing)),
        )
        .add_systems(Update, update_fps_text)
        .add_systems(
            Update,
            update_playing_input.run_if(in_state(AppState::Playing)),
        )
        .add_systems(OnExit(AppState::Playing), cleanup_playing)
        .add_systems(
            Update,
            update_result_input.run_if(in_state(AppState::Result)),
        )
        .add_systems(OnExit(AppState::Result), cleanup_result)
        .run();
}
// State-specific systems are provided by `state` module submodules.
