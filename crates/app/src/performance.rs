use std::time::{Duration, Instant};

pub const MAIN_LOOP_PERIOD: Duration = Duration::from_micros(50);
pub const RENDER_PERIOD: Duration = Duration::from_nanos(8_333_333);

pub struct PerformanceCounter {
    sample_started_at: Instant,
    loop_count: u64,
    render_count: u64,
    pub loops_per_second: f64,
    pub frames_per_second: f64,
}

impl PerformanceCounter {
    pub fn new() -> Self {
        Self {
            sample_started_at: Instant::now(),
            loop_count: 0,
            render_count: 0,
            loops_per_second: 0.0,
            frames_per_second: 0.0,
        }
    }

    pub fn record_loop(&mut self) -> bool {
        self.loop_count += 1;
        let elapsed = self.sample_started_at.elapsed();
        if elapsed < Duration::from_secs(1) {
            return false;
        }

        let seconds = elapsed.as_secs_f64();
        self.loops_per_second = self.loop_count as f64 / seconds;
        self.frames_per_second = self.render_count as f64 / seconds;
        self.loop_count = 0;
        self.render_count = 0;
        self.sample_started_at = Instant::now();
        true
    }

    pub fn record_render(&mut self) {
        self.render_count += 1;
    }
}
