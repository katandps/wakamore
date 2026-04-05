mod component;

use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy::winit::WinitSettings;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use component::{animate_note, handle_lane_input, setup_fps, setup_judge_line, setup_note, update_fps_text, FpsHistory};

fn main() {
    App::new()
        .insert_resource(WinitSettings::game())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                present_mode: PresentMode::AutoNoVsync,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .init_resource::<FpsHistory>()
        .add_systems(Startup, setup)
        .add_systems(Startup, setup_judge_line)
        .add_systems(Update, (update_fps_text, handle_lane_input, animate_note))
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d::default());
    setup_fps(&mut commands);
    setup_note(&mut commands);
}
