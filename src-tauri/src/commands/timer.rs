use crate::AppState;
use tauri::State;

#[tauri::command]
pub fn start_timer(state: State<'_, AppState>) {
    let engine = state.timer_engine.lock().unwrap();
    let slots = state.current_slots.lock().unwrap();
    for (idx, _) in slots.iter().enumerate() {
        engine.send(crate::core::EngineCommand::StartSlot(idx));
    }
}

#[tauri::command]
pub fn pause_timer(state: State<'_, AppState>) {
    let engine = state.timer_engine.lock().unwrap();
    let slots = state.current_slots.lock().unwrap();
    for (idx, _) in slots.iter().enumerate() {
        engine.send(crate::core::EngineCommand::PauseSlot(idx));
    }
}

#[tauri::command]
pub fn reset_timer(state: State<'_, AppState>) {
    let engine = state.timer_engine.lock().unwrap();
    engine.send(crate::core::EngineCommand::ResetAll);
}

#[tauri::command]
pub fn get_timer_snapshot(state: State<'_, AppState>) -> crate::core::timer_engine::TickPayload {
    let engine = state.timer_engine.lock().unwrap();
    engine.snapshot()
}

#[tauri::command]
pub fn start_slot(state: State<'_, AppState>, index: usize) {
    let engine = state.timer_engine.lock().unwrap();
    engine.send(crate::core::EngineCommand::StartSlot(index));
}

#[tauri::command]
pub fn pause_slot(state: State<'_, AppState>, index: usize) {
    let engine = state.timer_engine.lock().unwrap();
    engine.send(crate::core::EngineCommand::PauseSlot(index));
}

#[tauri::command]
pub fn reset_slot(state: State<'_, AppState>, index: usize) {
    let engine = state.timer_engine.lock().unwrap();
    engine.send(crate::core::EngineCommand::ResetSlot(index));
}

/// Toggle a slot: idle/triggered -> start, running -> reset, paused -> start
#[tauri::command]
pub fn toggle_slot(state: State<'_, AppState>, index: usize) {
    let engine = state.timer_engine.lock().unwrap();
    engine.send(crate::core::EngineCommand::ToggleSlot(index));
}

/// Trigger a hotkey: find bound slots and act according to the rules:
/// - Single bound slot: toggle (start/reset)
/// - Multiple bound slots (shared): start the next idle slot in order, never reset
#[tauri::command]
pub fn trigger_hotkey(state: State<'_, AppState>, hotkey: String) {
    let slots = state.current_slots.lock().unwrap();

    // Find all slot indices bound to this hotkey
    let mut bound_indices: Vec<usize> = Vec::new();
    for (idx, slot) in slots.iter().enumerate() {
        if let Some(ref hk) = slot.hotkey {
            if hk == &hotkey {
                bound_indices.push(idx);
            }
        }
    }

    if bound_indices.is_empty() {
        return;
    }

    let engine = state.timer_engine.lock().unwrap();

    if bound_indices.len() == 1 {
        // Single slot: toggle (start / reset)
        engine.send(crate::core::EngineCommand::ToggleSlot(bound_indices[0]));
    } else {
        // Multiple slots sharing the same hotkey: start the first idle slot in order
        engine.send(crate::core::EngineCommand::StartSlotsBatch(bound_indices));
    }
}
