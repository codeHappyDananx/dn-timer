use crate::AppState;
use tauri::{AppHandle, Manager, State};

#[tauri::command]
pub fn close_window(app: AppHandle, state: State<'_, AppState>) {
    if let Some(_window) = app.get_webview_window("main") {
        let _ = crate::platform::window::save_window_position(&app, &state.db);
    }
    app.exit(0);
}

#[tauri::command]
pub fn minimize_window(app: AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

#[tauri::command]
pub fn show_window(app: AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[tauri::command(rename_all = "snake_case")]
pub fn frontend_ready(app: AppHandle, state: State<'_, AppState>, simple_mode: bool) {
    if let Some(window) = app.get_webview_window("main") {
        if simple_mode {
            let slots = state.current_slots.lock().unwrap();
            let width = estimate_simple_mode_width(slots.iter().map(|slot| slot.name.as_str()));
            let scale = read_simple_mode_scale(&state).unwrap_or(0.88);
            let _ = crate::platform::window::enter_simple_mode(&app, slots.len(), width, scale);
        } else {
            if let Ok(db) = state.db.lock() {
                let _ = crate::platform::window::restore_window_position(&app, &*db);
            }
        }
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[tauri::command(rename_all = "snake_case")]
pub fn set_always_on_top(app: AppHandle, state: State<'_, AppState>, always_on_top: bool) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_always_on_top(always_on_top);
    }
    if let Ok(mut db) = state.db.lock() {
        let _ = db.set_config("always_on_top", &always_on_top.to_string());
    }
}

#[tauri::command]
pub fn get_always_on_top(state: State<'_, AppState>) -> Result<bool, String> {
    let db = state.db.lock().unwrap();
    match db.get_config("always_on_top") {
        Ok(Some(v)) => Ok(v.parse().unwrap_or(false)),
        _ => Ok(false),
    }
}

#[tauri::command]
pub fn get_window_position(app: AppHandle) -> Result<Option<(i32, i32, u32, u32)>, String> {
    if let Some(window) = app.get_webview_window("main") {
        let pos = window.outer_position().map_err(|e| e.to_string())?;
        let size = window.outer_size().map_err(|e| e.to_string())?;
        return Ok(Some((pos.x, pos.y, size.width, size.height)));
    }
    Ok(None)
}

#[tauri::command]
pub fn resize_window(app: AppHandle, width: f64, height: f64) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        crate::platform::window::set_content_window_size(&window, width, height)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub fn begin_window_drag(app: AppHandle, dock_on_release: bool) -> Result<(), String> {
    crate::platform::window::begin_window_drag(&app, dock_on_release).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn dock_window(app: AppHandle) -> Result<(), String> {
    crate::platform::window::dock_window(&app).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn undock_window(app: AppHandle) -> Result<(), String> {
    crate::platform::window::undock_window(&app).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_startup(enable: bool) -> Result<(), String> {
    crate::platform::window::set_startup(enable).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn is_startup_enabled() -> bool {
    crate::platform::window::is_startup_enabled()
}

#[tauri::command]
pub fn is_docked_window() -> bool {
    crate::platform::window::is_docked()
}

#[tauri::command]
pub fn enter_simple_mode(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let slots = state.current_slots.lock().unwrap();
    let width = estimate_simple_mode_width(slots.iter().map(|slot| slot.name.as_str()));
    let scale = read_simple_mode_scale(&state).unwrap_or(0.88);
    crate::platform::window::enter_simple_mode(&app, slots.len(), width, scale)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn exit_simple_mode(app: AppHandle) -> Result<(), String> {
    crate::platform::window::exit_simple_mode(&app).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_simple_mode(state: State<'_, AppState>, enabled: bool) -> Result<(), String> {
    if let Ok(mut db) = state.db.lock() {
        let _ = db.set_config("simple_mode", &enabled.to_string());
    }
    Ok(())
}

#[tauri::command]
pub fn get_simple_mode(state: State<'_, AppState>) -> Result<bool, String> {
    let db = state.db.lock().unwrap();
    match db.get_config("simple_mode") {
        Ok(Some(v)) => Ok(v.parse().unwrap_or(false)),
        _ => Ok(false),
    }
}

#[tauri::command(rename_all = "snake_case")]
pub fn set_simple_mode_scale(
    app: AppHandle,
    state: State<'_, AppState>,
    scale: f64,
) -> Result<f64, String> {
    let scale = crate::platform::window::normalize_simple_scale(scale);
    if let Ok(mut db) = state.db.lock() {
        let _ = db.set_config("simple_mode_scale", &scale.to_string());
    }

    let slots = state.current_slots.lock().unwrap();
    let width = estimate_simple_mode_width(slots.iter().map(|slot| slot.name.as_str()));
    crate::platform::window::enter_simple_mode(&app, slots.len(), width, scale)
        .map_err(|e| e.to_string())?;
    Ok(scale)
}

#[tauri::command]
pub fn get_simple_mode_scale(state: State<'_, AppState>) -> Result<f64, String> {
    Ok(read_simple_mode_scale(&state).unwrap_or(0.88))
}

fn read_simple_mode_scale(state: &State<'_, AppState>) -> Option<f64> {
    let db = state.db.lock().ok()?;
    let value = db.get_config("simple_mode_scale").ok().flatten()?;
    value
        .parse::<f64>()
        .ok()
        .map(crate::platform::window::normalize_simple_scale)
}

fn estimate_simple_mode_width<'a>(names: impl Iterator<Item = &'a str>) -> f64 {
    let max_name_width = names
        .map(|name| {
            name.chars()
                .map(|ch| if ch.is_ascii() { 7.0 } else { 13.0 })
                .sum::<f64>()
        })
        .fold(0.0, f64::max);
    (max_name_width + 330.0).clamp(430.0, 760.0)
}
