use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

#[derive(Component)]
pub struct FpsText;

#[derive(Resource, Default)]
pub struct FpsHistory {
    pub samples: Vec<(f64, f64)>,
}

pub fn setup_fps(mut commands: Commands) {
    commands.spawn((
        Text::new("FPS: 0"),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::srgb(0.0, 1.0, 0.0)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        FpsText,
    ));
}

pub fn update_fps_text(
    diagnostics: Res<DiagnosticsStore>,
    time: Res<Time>,
    mut history: ResMut<FpsHistory>,
    mut query: Query<&mut Text, With<FpsText>>,
) {
    if let Ok(mut text) = query.single_mut() {
        if let Some(fps_diagnostic) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
            let current_fps = fps_diagnostic.smoothed().unwrap_or(0.0);

            let now = time.elapsed_secs_f64();
            history.samples.push((now, current_fps));
            history.samples.retain(|(t, _)| now - *t <= 1.0);

            let sample_count = history.samples.len() as f64;
            let avg_fps = if sample_count > 0.0 {
                history.samples.iter().map(|(_, fps)| *fps).sum::<f64>() / sample_count
            } else {
                0.0
            };
            let max_fps = history
                .samples
                .iter()
                .map(|(_, fps)| *fps)
                .fold(0.0_f64, f64::max);

            text.0 = format!(
                "FPS: {:.1}\nAvg: {:.1}\nMax: {:.1}",
                current_fps, avg_fps, max_fps
            );
        }
    }
}
