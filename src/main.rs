mod component;

use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy::winit::WinitSettings;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use component::{setup_fps, update_fps_text, FpsHistory};

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
        .add_systems(Update, update_fps_text)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d::default());
    setup_fps(commands);
}
