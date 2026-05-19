use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub warn_seconds: u32,
    pub warn_sound_enabled: bool,
    pub warn_sound_key: String,
    pub warn_sound_repeat: String,
    pub always_on_top: bool,
    pub opacity: f64,
    pub auto_start: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            warn_seconds: 10,
            warn_sound_enabled: true,
            warn_sound_key: "soft_chime".to_string(),
            warn_sound_repeat: "1".to_string(),
            always_on_top: true,
            opacity: 1.0,
            auto_start: false,
        }
    }
}
