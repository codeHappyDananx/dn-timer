use crate::models::preset::{PresetType, TimerPreset};
use crate::AppState;
use serde::Deserialize;
use tauri::State;

#[derive(Debug, Deserialize)]
pub struct PresetRequest {
    name: String,
    desc: Option<String>,
    preset_type: PresetType,
}

#[tauri::command]
pub fn list_presets(state: State<'_, AppState>) -> Result<Vec<TimerPreset>, String> {
    let db = state.db.lock().unwrap();
    db.list_presets().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_current_preset_id(state: State<'_, AppState>) -> Option<String> {
    state.current_preset_id.lock().unwrap().clone()
}

#[tauri::command]
pub fn create_preset(state: State<'_, AppState>, data: PresetRequest) -> Result<(), String> {
    let mut db = state.db.lock().unwrap();
    let mut preset = TimerPreset::new(&data.name, data.preset_type);
    preset.desc = data.desc;
    db.save_preset(&preset).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_preset(
    state: State<'_, AppState>,
    id: String,
    data: PresetRequest,
) -> Result<(), String> {
    let mut db = state.db.lock().unwrap();
    let mut preset = db
        .get_preset(&id)
        .map_err(|e| e.to_string())?
        .ok_or("Preset not found")?;
    preset.name = data.name;
    preset.desc = data.desc;
    preset.preset_type = data.preset_type;
    preset.updated_at = chrono::Utc::now();
    db.save_preset(&preset).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_preset(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let mut db = state.db.lock().unwrap();
    db.delete_preset(&id).map_err(|e| e.to_string())?;

    let is_current = {
        let current = state.current_preset_id.lock().unwrap();
        current.as_ref() == Some(&id)
    };

    if is_current {
        {
            let mut manager = state.hotkey_manager.lock().unwrap();
            manager.update_bindings(std::collections::HashMap::new());
        }
        {
            let engine = state.timer_engine.lock().unwrap();
            engine.send(crate::core::EngineCommand::LoadSlots(Vec::new()));
        }
        {
            let mut current_slots = state.current_slots.lock().unwrap();
            current_slots.clear();
        }
        {
            let mut current = state.current_preset_id.lock().unwrap();
            *current = None;
        }
    }

    Ok(())
}

#[tauri::command]
pub fn select_preset(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    let preset = db
        .get_preset(&id)
        .map_err(|e| e.to_string())?
        .ok_or("Preset not found")?;

    let slots = match &preset.preset_type {
        crate::models::preset::PresetType::Single {
            first,
            loop_interval,
            warn_seconds,
            warn_text,
            hotkey,
            bar_color,
            text_color,
            ..
        } => {
            vec![crate::models::preset::TimerSlotDef {
                id: preset.id.clone(),
                name: preset.name.clone(),
                first: *first,
                loop_interval: *loop_interval,
                warn_seconds: *warn_seconds,
                warn_text: warn_text.clone(),
                hotkey: hotkey.clone(),
                bar_color: bar_color.clone(),
                text_color: text_color.clone(),
            }]
        }
        crate::models::preset::PresetType::Multi { slots, .. } => slots.clone(),
    };

    // Build hotkey bindings from slot definitions
    let mut bindings: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    for slot in slots.iter() {
        if let Some(ref hk) = slot.hotkey {
            bindings.insert(hk.clone(), hk.clone());
        }
    }

    // Update hotkey manager bindings
    {
        let mut manager = state.hotkey_manager.lock().unwrap();
        manager.update_bindings(bindings);
    }

    // Load slots into engine
    let engine = state.timer_engine.lock().unwrap();
    engine.send(crate::core::EngineCommand::LoadSlots(slots.clone()));

    // Cache current slots and preset id
    {
        let mut current_slots = state.current_slots.lock().unwrap();
        *current_slots = slots;
    }
    {
        let mut current = state.current_preset_id.lock().unwrap();
        *current = Some(id);
    }

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub fn update_slot_hotkey(
    state: State<'_, AppState>,
    preset_id: String,
    slot_index: usize,
    hotkey: Option<String>,
) -> Result<(), String> {
    update_slot_config_impl(state, preset_id, slot_index, hotkey, None, None)
}

#[tauri::command(rename_all = "snake_case")]
pub fn update_slot_config(
    state: State<'_, AppState>,
    preset_id: String,
    slot_index: usize,
    hotkey: Option<String>,
    bar_color: Option<String>,
    text_color: Option<String>,
) -> Result<(), String> {
    update_slot_config_impl(state, preset_id, slot_index, hotkey, bar_color, text_color)
}

fn update_slot_config_impl(
    state: State<'_, AppState>,
    preset_id: String,
    slot_index: usize,
    hotkey: Option<String>,
    bar_color: Option<String>,
    text_color: Option<String>,
) -> Result<(), String> {
    let mut db = state.db.lock().unwrap();
    let mut preset = db
        .get_preset(&preset_id)
        .map_err(|e| e.to_string())?
        .ok_or("Preset not found")?;

    match &mut preset.preset_type {
        PresetType::Single {
            hotkey: slot_hotkey,
            bar_color: slot_bar_color,
            text_color: slot_text_color,
            ..
        } => {
            if slot_index != 0 {
                return Err("Slot index out of range".to_string());
            }
            *slot_hotkey = hotkey;
            if let Some(c) = bar_color {
                *slot_bar_color = Some(c);
            }
            if let Some(c) = text_color {
                *slot_text_color = Some(c);
            }
        }
        PresetType::Multi { slots, .. } => {
            if let Some(slot) = slots.get_mut(slot_index) {
                slot.hotkey = hotkey;
                if let Some(c) = bar_color {
                    slot.bar_color = Some(c);
                }
                if let Some(c) = text_color {
                    slot.text_color = Some(c);
                }
            } else {
                return Err("Slot index out of range".to_string());
            }
        }
    }

    preset.updated_at = chrono::Utc::now();
    db.save_preset(&preset).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn next_preset(state: State<'_, AppState>) -> Result<(), String> {
    let current_id = {
        let current = state.current_preset_id.lock().unwrap();
        current.clone()
    };

    let next_id = {
        let db = state.db.lock().unwrap();
        let presets = db.list_presets().map_err(|e| e.to_string())?;
        if presets.is_empty() {
            return Ok(());
        }

        if let Some(current_id) = current_id {
            let current_index = presets.iter().position(|p| p.id == current_id);
            match current_index {
                Some(idx) => {
                    let next_idx = (idx + 1) % presets.len();
                    presets[next_idx].id.clone()
                }
                None => presets[0].id.clone(),
            }
        } else {
            presets[0].id.clone()
        }
    };

    select_preset(state, next_id)
}
