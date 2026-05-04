use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum Lane7S {
    /// Lane 1 (left-most)
    Lane1 = 0,
    /// Lane 2
    Lane2 = 1,
    /// Lane 3
    Lane3 = 2,
    /// Lane 4
    Lane4 = 3,
    /// Lane 5
    Lane5 = 4,
    /// Lane 6
    Lane6 = 5,
    /// Lane 7
    Lane7 = 6,
    /// Special lane (Scratch)
    Scratch = 7,
}

#[derive(Clone, Debug)]
pub struct ChartNote {
    pub lane: Lane7S,
    pub time_from_start_secs: f32,
}

#[derive(Resource, Default)]
pub struct NoteChart {
    pub notes: Vec<ChartNote>,
}

impl NoteChart {
    pub fn demo() -> Self {
        Self {
            notes: vec![
                ChartNote {
                    lane: Lane7S::Lane1,
                    time_from_start_secs: 0.50,
                },
                ChartNote {
                    lane: Lane7S::Lane4,
                    time_from_start_secs: 0.90,
                },
                ChartNote {
                    lane: Lane7S::Lane6,
                    time_from_start_secs: 1.20,
                },
                ChartNote {
                    lane: Lane7S::Scratch,
                    time_from_start_secs: 1.70,
                },
            ],
        }
    }

    /// Create a NoteChart from a parsed BMS structure.
    ///
    /// This performs a simple conversion assuming a fixed BPM (from #BPM header)
    /// and 4/4 measures. Channel→lane mapping is conservative: channels 11..17
    /// map to `Lane1`..`Lane7`, and channels 18/19 map to `Scratch`.
    pub fn from_bms(b: &bms::Bms) -> Self {
        // Lane7S is defined in this module
        let mut notes = Vec::new();

        // Get BPM from header, default to 120.0 if missing/invalid
        let bpm = b
            .header(&bms::HeaderKey::Bpm)
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(120.0_f32);
        let secs_per_beat = 60.0_f32 / bpm;
        let beats_per_measure = 4.0_f32; // assume 4/4
        let measure_secs = beats_per_measure * secs_per_beat;

        // helper to map channel to lane
        let map_channel = |ch: u8| -> Option<Lane7S> {
            match ch {
                11 => Some(Lane7S::Lane1),
                12 => Some(Lane7S::Lane2),
                13 => Some(Lane7S::Lane3),
                14 => Some(Lane7S::Lane4),
                15 => Some(Lane7S::Lane5),
                16 => Some(Lane7S::Lane6),
                17 => Some(Lane7S::Lane7),
                18 | 19 => Some(Lane7S::Scratch),
                _ => None,
            }
        };

        for (measure, channels) in &b.measures {
            // BMS measures typically start at 1 for #001; treat measure 1 as time 0.0
            let measure_idx = (*measure).saturating_sub(1) as f32;
            let measure_start = measure_idx * measure_secs;

            for ch in channels {
                if let Some(lane) = map_channel(ch.channel) {
                    let total = ch.data.len();
                    if total == 0 {
                        continue;
                    }
                    for (i, item) in ch.data.iter().enumerate() {
                        if item.is_none() {
                            continue;
                        }
                        let frac = (i as f32) / (total as f32);
                        let t = measure_start + frac * measure_secs;
                        notes.push(ChartNote {
                            lane,
                            time_from_start_secs: t,
                        });
                    }
                }
            }
        }

        // sort by time
        notes.sort_by(|a, b| {
            a.time_from_start_secs
                .partial_cmp(&b.time_from_start_secs)
                .unwrap()
        });

        NoteChart { notes }
    }
}

#[derive(Resource, Default)]
pub struct ScoreSummary {
    pub pg: u32,
    pub gr: u32,
    pub miss: u32,
    pub score: u32,
}
