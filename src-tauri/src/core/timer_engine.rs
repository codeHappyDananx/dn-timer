use crossbeam::channel::{bounded, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};

use super::timer_state::{SlotEvent, TimerSlotRuntime};
use crate::models::preset::{SlotStatus, TimerSlotDef};

#[derive(Debug, Clone)]
pub enum EngineCommand {
    StartSlot(usize),
    PauseSlot(usize),
    ResetSlot(usize),
    ToggleSlot(usize),
    StartSlotsBatch(Vec<usize>),
    ResetAll,
    LoadSlots(Vec<TimerSlotDef>),
    UpdateWarnConfig(WarnConfig),
}

#[derive(Debug, Clone)]
pub struct WarnConfig {
    pub sound_enabled: bool,
    pub sound_key: String,
    pub sound_repeat: String, // "1", "2", "3", "continuous"
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SlotSnapshot {
    id: String,
    name: String,
    elapsed_ms: u64,
    remaining_ms: u64,
    target_ms: u64,
    status: String,
    loop_count: u32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TickPayload {
    slots: Vec<SlotSnapshot>,
    global_status: String,
}

pub struct TimerEngine {
    command_tx: Sender<EngineCommand>,
    latest_snapshot: Arc<Mutex<TickPayload>>,
    _worker: thread::JoinHandle<()>,
}

impl TimerEngine {
    pub fn new(app_handle: AppHandle) -> Self {
        let (cmd_tx, cmd_rx) = bounded::<EngineCommand>(32);
        let latest_snapshot = Arc::new(Mutex::new(TickPayload {
            slots: Vec::new(),
            global_status: "idle".to_string(),
        }));
        let worker_snapshot = latest_snapshot.clone();
        let worker = thread::spawn(move || {
            Self::run_loop(cmd_rx, app_handle, worker_snapshot);
        });

        Self {
            command_tx: cmd_tx,
            latest_snapshot,
            _worker: worker,
        }
    }

    pub fn send(&self, cmd: EngineCommand) {
        let _ = self.command_tx.send(cmd);
    }

    pub fn snapshot(&self) -> TickPayload {
        self.latest_snapshot.lock().unwrap().clone()
    }

    fn run_loop(
        cmd_rx: Receiver<EngineCommand>,
        app_handle: AppHandle,
        latest_snapshot: Arc<Mutex<TickPayload>>,
    ) {
        const TICK_INTERVAL: Duration = Duration::from_millis(50);
        let mut slots: Vec<TimerSlotRuntime> = Vec::new();
        let mut last_tick = Instant::now();
        let mut warn_config: Option<WarnConfig> = None;
        let mut continuous_warn_last_play: Option<Instant> = None;

        loop {
            // Process commands
            while let Ok(cmd) = cmd_rx.try_recv() {
                match cmd {
                    EngineCommand::LoadSlots(defs) => {
                        slots = defs
                            .into_iter()
                            .enumerate()
                            .map(|(i, def)| TimerSlotRuntime::new(i, def))
                            .collect();
                    }
                    EngineCommand::StartSlot(idx) => {
                        if let Some(slot) = slots.get_mut(idx) {
                            slot.start();
                        }
                    }
                    EngineCommand::PauseSlot(idx) => {
                        if let Some(slot) = slots.get_mut(idx) {
                            slot.pause();
                        }
                    }
                    EngineCommand::ResetSlot(idx) => {
                        if let Some(slot) = slots.get_mut(idx) {
                            slot.restart();
                        }
                    }
                    EngineCommand::ToggleSlot(idx) => {
                        if let Some(slot) = slots.get_mut(idx) {
                            match slot.status {
                                SlotStatus::Idle | SlotStatus::Triggered => slot.start(),
                                SlotStatus::Running => slot.restart(),
                                SlotStatus::Paused => slot.start(),
                                _ => {}
                            }
                        }
                    }
                    EngineCommand::StartSlotsBatch(indices) => {
                        for idx in indices {
                            if let Some(slot) = slots.get_mut(idx) {
                                if slot.status == SlotStatus::Idle
                                    || slot.status == SlotStatus::Triggered
                                {
                                    slot.start();
                                    break; // Only start the first idle slot
                                }
                            }
                        }
                    }
                    EngineCommand::ResetAll => {
                        for slot in &mut slots {
                            slot.reset();
                        }
                    }
                    EngineCommand::UpdateWarnConfig(config) => {
                        warn_config = Some(config);
                    }
                }
            }

            let now = Instant::now();
            let delta = now.duration_since(last_tick);
            last_tick = now;

            // Update running slots
            let mut has_running = false;
            let mut has_active_warning = false;

            for slot in &mut slots {
                let events = slot.tick(delta);

                if slot.status == SlotStatus::Running {
                    has_running = true;
                }

                for event in events {
                    match event {
                        SlotEvent::Warn => {
                            let _ = app_handle.emit(
                                "timer:warn",
                                serde_json::json!({
                                    "slot_index": slot.index,
                                    "slot_id": &slot.id,
                                }),
                            );

                            if let Some(ref config) = warn_config {
                                if config.sound_enabled {
                                    let repeat_count: usize = match config.sound_repeat.as_str() {
                                        "1" => 1,
                                        "2" => 2,
                                        "3" => 3,
                                        "continuous" => {
                                            continuous_warn_last_play = Some(Instant::now());
                                            1
                                        }
                                        _ => 1,
                                    };
                                    for _ in 0..repeat_count {
                                        crate::utils::sound::play_alert_sound(&config.sound_key);
                                    }
                                }
                            }
                        }
                        SlotEvent::Trigger { auto_continue } => {
                            let _ = app_handle.emit(
                                "timer:trigger",
                                serde_json::json!({
                                    "slot_index": slot.index,
                                    "slot_id": &slot.id,
                                    "loop_count": slot.loop_count,
                                    "auto_continue": auto_continue,
                                }),
                            );
                            continuous_warn_last_play = None;
                        }
                    }
                }

                // Track if any slot is currently in warning phase
                if slot.status == SlotStatus::Running
                    && slot.warn_fired
                    && slot.elapsed < slot.target
                {
                    has_active_warning = true;
                }
            }

            // Handle continuous warning sound
            if has_active_warning {
                if let Some(ref config) = warn_config {
                    if config.sound_enabled && config.sound_repeat == "continuous" {
                        if let Some(last) = continuous_warn_last_play {
                            if now.duration_since(last) >= Duration::from_secs(1) {
                                crate::utils::sound::play_alert_sound(&config.sound_key);
                                continuous_warn_last_play = Some(now);
                            }
                        }
                    }
                }
            } else {
                continuous_warn_last_play = None;
            }

            // Build snapshot and emit
            let snapshots: Vec<SlotSnapshot> = slots
                .iter()
                .map(|s| {
                    let remaining = s.target.saturating_sub(s.elapsed);
                    let phase_target = if s.loop_count > 0 && s.loop_interval > Duration::ZERO {
                        s.loop_interval
                    } else {
                        s.initial_target
                    };
                    let phase_elapsed = phase_target.saturating_sub(remaining);

                    SlotSnapshot {
                        id: s.id.clone(),
                        name: s.name.clone(),
                        elapsed_ms: phase_elapsed.as_millis() as u64,
                        remaining_ms: remaining.as_millis() as u64,
                        target_ms: phase_target.as_millis() as u64,
                        status: if s.status == SlotStatus::Running && s.warn_fired {
                            "warning".to_string()
                        } else {
                            format!("{:?}", s.status).to_lowercase()
                        },
                        loop_count: s.loop_count,
                    }
                })
                .collect();

            let global_status = if has_running {
                "running"
            } else if slots.iter().any(|s| s.status == SlotStatus::Paused) {
                "paused"
            } else {
                "idle"
            };

            let payload = TickPayload {
                slots: snapshots,
                global_status: global_status.to_string(),
            };

            {
                let mut latest = latest_snapshot.lock().unwrap();
                *latest = payload.clone();
            }

            let _ = app_handle.emit("timer:tick", payload);

            // Sleep until next tick
            let sleep_until = last_tick + TICK_INTERVAL;
            let now = Instant::now();
            if sleep_until > now {
                thread::sleep(sleep_until - now);
            }
        }
    }
}
