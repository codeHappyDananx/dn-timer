use crate::models::config::AppConfig;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub fn get_warn_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    let db = state.db.lock().unwrap();
    db.get_warn_config().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_warn_config(state: State<'_, AppState>, config: AppConfig) -> Result<(), String> {
    let mut db = state.db.lock().unwrap();
    db.set_warn_config(&config).map_err(|e| e.to_string())?;

    // 同步更新引擎中的 warn_config
    let engine = state.timer_engine.lock().unwrap();
    engine.send(crate::core::EngineCommand::UpdateWarnConfig(
        crate::core::WarnConfig {
            sound_enabled: config.warn_sound_enabled,
            sound_key: config.warn_sound_key.clone(),
            sound_repeat: config.warn_sound_repeat.clone(),
        },
    ));

    Ok(())
}
