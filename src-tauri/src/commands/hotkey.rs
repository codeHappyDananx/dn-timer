use crate::AppState;
use std::collections::HashMap;
use tauri::State;

#[tauri::command]
pub fn get_hotkey_bindings(state: State<'_, AppState>) -> Result<HashMap<String, String>, String> {
    let db = state.db.lock().unwrap();
    db.get_hotkey_bindings().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_hotkey_bindings(
    state: State<'_, AppState>,
    bindings: HashMap<String, String>,
) -> Result<(), String> {
    let mut db = state.db.lock().unwrap();
    db.set_hotkey_bindings(&bindings)
        .map_err(|e| e.to_string())?;

    let mut manager = state.hotkey_manager.lock().unwrap();
    manager.update_bindings(bindings);

    Ok(())
}
