use std::time::{Duration, Instant};

/// メインループ・描画の起動タイミングと再描画要求の保留状態をまとめる。
pub struct LoopManager {
    pub next_loop_at: Instant,
    period: Duration,
}

impl LoopManager {
    pub fn new(now: Instant, period: Duration) -> Self {
        Self {
            next_loop_at: now,
            period,
        }
    }

    pub fn next_loop_is_came(&self, now: Instant) -> bool {
        now >= self.next_loop_at
    }

    pub fn update_loop_state(&mut self, now: Instant) {
        let next = self.next_loop_at + self.period;
        let next = if next <= now { now + self.period } else { next };
        self.next_loop_at = next;
    }
}
