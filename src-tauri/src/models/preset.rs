use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerSlotDef {
    pub id: String,
    pub name: String,
    pub first: u32,
    pub loop_interval: u32,
    pub warn_seconds: u32,
    pub warn_text: Option<String>,
    pub hotkey: Option<String>,
    pub bar_color: Option<String>,
    pub text_color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PresetType {
    Single {
        first: u32,
        loop_interval: u32,
        followups: Vec<u32>,
        warn_seconds: u32,
        warn_text: Option<String>,
        #[serde(default)]
        hotkey: Option<String>,
        #[serde(default)]
        bar_color: Option<String>,
        #[serde(default)]
        text_color: Option<String>,
    },
    Multi {
        slots: Vec<TimerSlotDef>,
        sequential: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerPreset {
    pub id: String,
    pub name: String,
    pub desc: Option<String>,
    pub hint: Option<String>,
    pub note: Option<String>,
    pub preset_type: PresetType,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_builtin: bool,
}

impl TimerPreset {
    pub fn new(name: &str, preset_type: PresetType) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            desc: None,
            hint: None,
            note: None,
            preset_type,
            created_at: now,
            updated_at: now,
            is_builtin: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SlotStatus {
    Idle,
    Running,
    Paused,
    Warning,
    Triggered,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct TimerSlotState {
    pub id: String,
    pub name: String,
    pub status: SlotStatus,
    pub elapsed_ms: u64,
    pub target_ms: u64,
    pub remaining_ms: u64,
    pub loop_count: u32,
    pub warn_fired: bool,
}
