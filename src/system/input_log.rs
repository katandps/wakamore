use bevy::prelude::*;
use common::{InputLog, LaneJudgementEvent, LastRawByLane, LogEntry};

pub fn record_judgement_to_log(
    mut judgement_reader: MessageReader<LaneJudgementEvent>,
    time: Res<Time>,
    mut input_log: ResMut<InputLog>,
    last_raw: Res<LastRawByLane>,
) {
    for ev in judgement_reader.read() {
        let raw = last_raw.0.get(&ev.lane_index).cloned();
        input_log.0.push(LogEntry {
            timestamp: time.elapsed_secs_f64(),
            lane_index: ev.lane_index,
            raw,
            judgement: Some(ev.kind),
        });
    }
}
