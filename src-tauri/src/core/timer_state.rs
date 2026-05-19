use crate::models::preset::{SlotStatus, TimerSlotDef};
use std::time::Duration;

pub struct TimerSlotRuntime {
    pub index: usize,
    pub id: String,
    pub name: String,
    pub status: SlotStatus,
    pub elapsed: Duration,
    pub initial_target: Duration,
    pub target: Duration,
    pub loop_interval: Duration,
    pub loop_count: u32,
    pub warn_seconds: u32,
    pub warn_fired: bool,
}

pub enum SlotEvent {
    Warn,
    Trigger { auto_continue: bool },
}

impl TimerSlotRuntime {
    pub fn new(index: usize, def: TimerSlotDef) -> Self {
        Self {
            index,
            id: def.id,
            name: def.name,
            status: SlotStatus::Idle,
            elapsed: Duration::ZERO,
            initial_target: Duration::from_secs(def.first as u64),
            target: Duration::from_secs(def.first as u64),
            loop_interval: Duration::from_secs(def.loop_interval as u64),
            loop_count: 0,
            warn_seconds: def.warn_seconds,
            warn_fired: false,
        }
    }

    pub fn start(&mut self) {
        if self.status == SlotStatus::Paused {
            self.status = SlotStatus::Running;
        } else if self.status == SlotStatus::Idle || self.status == SlotStatus::Triggered {
            self.status = SlotStatus::Running;
            self.elapsed = Duration::ZERO;
            self.warn_fired = false;
        }
    }

    pub fn pause(&mut self) {
        if self.status == SlotStatus::Running {
            self.status = SlotStatus::Paused;
        }
    }

    pub fn reset(&mut self) {
        self.status = SlotStatus::Idle;
        self.elapsed = Duration::ZERO;
        self.target = self.initial_target;
        self.loop_count = 0;
        self.warn_fired = false;
    }

    pub fn restart(&mut self) {
        self.reset();
        self.status = SlotStatus::Running;
    }

    pub fn tick(&mut self, delta: Duration) -> Vec<SlotEvent> {
        if self.status != SlotStatus::Running {
            return Vec::new();
        }

        self.elapsed += delta;
        let mut events = Vec::new();

        // Check warning
        if !self.warn_fired && self.target > Duration::ZERO {
            let warn_at = self
                .target
                .saturating_sub(Duration::from_secs(self.warn_seconds as u64));
            if self.elapsed >= warn_at {
                self.warn_fired = true;
                events.push(SlotEvent::Warn);
            }
        }

        // Check trigger
        if self.target > Duration::ZERO && self.elapsed >= self.target {
            self.loop_count += 1;
            if self.loop_interval > Duration::ZERO {
                self.target = self.elapsed + self.loop_interval;
                self.warn_fired = false;
                events.push(SlotEvent::Trigger {
                    auto_continue: true,
                });
            } else {
                self.status = SlotStatus::Triggered;
                events.push(SlotEvent::Trigger {
                    auto_continue: false,
                });
            }
        }

        events
    }
}
