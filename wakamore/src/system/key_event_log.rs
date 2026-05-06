//! キーイベントオーバーレイ: プレイ中に受け取った InputEvent を画面右側に時刻付きで表示する。

use bevy::prelude::*;
use common::InputEvent;
use std::collections::VecDeque;

const MAX_ENTRIES: usize = 20;
const ENTRY_LIFETIME_SECS: f32 = 6.0;
const OVERLAY_COLOR: Color = Color::srgb(0.6, 1.0, 0.2); // 黄緑

/// 最近の InputEvent を時刻付きで保持するリングバッファ。
#[derive(Resource, Default)]
pub struct KeyEventLog(pub VecDeque<(f32, String)>); // (elapsed_secs, ラベル)

/// キーイベントログを表示する UI テキストエンティティのマーカー。
#[derive(Component)]
pub struct KeyEventLogUi;

/// 起動時に UI テキストエンティティを生成する（全画面で常時表示）。
pub fn setup_key_event_log_ui(mut commands: Commands) {
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: 11.0,
            ..default()
        },
        TextColor(OVERLAY_COLOR),
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(8.0),
            top: Val::Px(8.0),
            // 折り返しが起きない十分な幅
            width: Val::Px(240.0),
            ..default()
        },
        KeyEventLogUi,
    ));
}

/// InputEvent を購読してログバッファに追記する。
pub fn record_key_events<E: InputEvent + std::fmt::Debug>(
    mut input_reader: MessageReader<E>,
    time: Res<Time>,
    mut log: ResMut<KeyEventLog>,
) {
    let now = time.elapsed_secs();
    for ev in input_reader.read() {
        let label = format!("{:?}", ev);
        log.0.push_back((now, label));
        if log.0.len() > MAX_ENTRIES {
            log.0.pop_front();
        }
    }
}

/// ログバッファの内容を UI テキストに反映し、古いエントリを削除する。
pub fn update_key_event_log_ui(
    time: Res<Time>,
    mut log: ResMut<KeyEventLog>,
    mut ui_q: Query<&mut Text, With<KeyEventLogUi>>,
) {
    let now = time.elapsed_secs();
    log.0.retain(|(t, _)| now - *t < ENTRY_LIFETIME_SECS);

    let Ok(mut text) = ui_q.single_mut() else {
        return;
    };

    let mut s = String::new();
    for (t, label) in log.0.iter().rev() {
        s.push_str(&format!("{:9.4}  {}\n", t, label));
    }
    text.0 = s;
}

// label is derived from Debug impl of each per-screen event enum.
