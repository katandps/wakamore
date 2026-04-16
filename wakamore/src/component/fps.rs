use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use std::collections::VecDeque;

#[derive(Component)]
pub struct FpsText;

#[derive(Resource, Default)]
pub struct FpsHistory {
    pub samples: VecDeque<(f64, f64)>,
}

impl FpsHistory {
    pub fn push_sample(&mut self, now: f64, fps: f64, window_secs: f64) {
        self.samples.push_back((now, fps));
        while let Some((t, _)) = self.samples.front() {
            if now - *t > window_secs {
                self.samples.pop_front();
            } else {
                break;
            }
        }
    }

    pub fn avg(&self) -> f64 {
        if self.samples.is_empty() {
            0.0
        } else {
            let sum = self.samples.iter().map(|(_, fps)| *fps).sum::<f64>();
            sum / (self.samples.len() as f64)
        }
    }

    pub fn max(&self) -> f64 {
        self.samples
            .iter()
            .map(|(_, fps)| *fps)
            .fold(0.0_f64, f64::max)
    }
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
            history.push_sample(now, current_fps, 1.0);

            let avg_fps = history.avg();
            let max_fps = history.max();

            text.0 = format!(
                "FPS: {:.1}\nAvg: {:.1}\nMax: {:.1}",
                current_fps, avg_fps, max_fps
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::FpsHistory;

    #[test]
    fn fps_history_push_and_stats() {
        let mut h = FpsHistory::default();
        h.push_sample(0.0, 60.0, 1.0);
        h.push_sample(0.5, 58.0, 1.0);
        h.push_sample(1.5, 30.0, 1.0); // should evict the first sample (0.0)
        assert_eq!(h.samples.len(), 2);
        let avg = h.avg();
        assert!((avg - ((58.0 + 30.0) / 2.0)).abs() < 1e-9);
        assert_eq!(h.max(), 58.0);
    }

    #[test]
    fn empty_history() {
        let h = FpsHistory::default();
        assert_eq!(h.avg(), 0.0);
        assert_eq!(h.max(), 0.0);
    }
}
