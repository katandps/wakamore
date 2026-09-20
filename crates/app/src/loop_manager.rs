use std::time::{Duration, Instant};

/// メインループ・描画の起動タイミングと再描画要求の保留状態をまとめる。
pub struct LoopManager {
    next_loop_at: Instant,
    render_pending: bool,
    period: Duration,
}

impl LoopManager {
    pub fn new(now: Instant, period: Duration) -> Self {
        Self {
            next_loop_at: now,
            render_pending: false,
            period,
        }
    }

    pub fn next_loop_is_came(&self, now: Instant) -> bool {
        now >= self.next_loop_at
    }

    pub fn render_is_pending(&self) -> bool {
        self.render_pending
    }

    pub fn render_set_pending(&mut self, pending: bool) {
        self.render_pending = pending;
    }

    pub fn next_wait_deadline(&self) -> Instant {
        self.next_loop_at.min(self.next_loop_at)
    }

    pub fn update_loop_state(&mut self, now: Instant) {
        self.next_loop_at = Self::next_deadline(now, self.next_loop_at, self.period);
    }

    fn next_deadline(now: Instant, previous: Instant, period: std::time::Duration) -> Instant {
        let next = previous + period;
        if next <= now { now + period } else { next }
    }
}
