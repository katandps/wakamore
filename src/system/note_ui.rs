use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::component::note::{JudgeLine, LaneJudgeText, LaneKeyIndicator};
use crate::component::{JUDGE_LINE_Y_FROM_BOTTOM, KEY_INDICATOR_DIAMETER, KEY_INDICATOR_Y_FROM_BOTTOM, JUDGEMENT_TEXT_Y_FROM_BOTTOM};

pub fn sync_lane_ui_layout(
    window_q: Query<&Window, With<PrimaryWindow>>,
    mut judge_line_q: Query<&mut Transform, With<JudgeLine>>,
    mut indicator_q: Query<(&LaneKeyIndicator, &mut Node), Without<LaneJudgeText>>,
    mut judge_text_q: Query<(&LaneJudgeText, &mut Node), Without<LaneKeyIndicator>>,
) {
    let Ok(window) = window_q.single() else { return; };
    let judge_line_y = -window.height() * 0.5 + JUDGE_LINE_Y_FROM_BOTTOM;
    for mut transform in &mut judge_line_q { transform.translation.y = judge_line_y; }
    for (indicator, mut node) in &mut indicator_q {
        let x = crate::component::lane_center_x(indicator.index);
        let screen_x = x + window.width() * 0.5;
        node.left = Val::Px(screen_x - KEY_INDICATOR_DIAMETER * 0.5);
        node.bottom = Val::Px(KEY_INDICATOR_Y_FROM_BOTTOM - KEY_INDICATOR_DIAMETER * 0.5);
    }
    for (judge_text, mut node) in &mut judge_text_q {
        let x = crate::component::lane_center_x(judge_text.index);
        let screen_x = x + window.width() * 0.5;
        node.left = Val::Px(screen_x - 16.0);
        node.bottom = Val::Px(JUDGEMENT_TEXT_Y_FROM_BOTTOM);
    }
}
