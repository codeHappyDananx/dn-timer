use crate::overlay::OverlayManager;
use tauri::{AppHandle, Manager};

#[tauri::command]
pub fn toggle_overlay(app: AppHandle) -> Result<(), String> {
    OverlayManager::toggle_overlay(&app).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_overlay_click_through(app: AppHandle, enable: bool) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("overlay") {
        OverlayManager::set_click_through(&window, enable).map_err(|e| e.to_string())?;
    }
    Ok(())
}
