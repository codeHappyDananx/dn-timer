use crate::models::character::{Character, ClearRecord, DungeonDef};
use crate::AppState;
use tauri::State;

#[tauri::command]
pub fn list_characters(state: State<'_, AppState>) -> Result<Vec<Character>, String> {
    let db = state.db.lock().unwrap();
    db.list_characters().map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn add_character(
    state: State<'_, AppState>,
    name: String,
    server: Option<String>,
    class_key: Option<String>,
) -> Result<(), String> {
    let mut db = state.db.lock().unwrap();
    let character = Character::new(&name, server.as_deref(), class_key.as_deref());
    db.add_character(&character).map_err(|e| e.to_string())?;

    // Initialize clear records for all dungeons
    let dungeons = db.list_dungeon_defs().map_err(|e| e.to_string())?;
    let week_start = chrono::Utc::now().naive_utc().date();
    for dungeon in dungeons {
        db.update_clear_record(
            &character.id,
            &dungeon.id,
            0,
            dungeon.max_clears,
            week_start,
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub fn update_character_class(
    state: State<'_, AppState>,
    id: String,
    class_key: Option<String>,
) -> Result<(), String> {
    let mut db = state.db.lock().unwrap();
    db.update_character_class(&id, class_key.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn update_character_note(
    state: State<'_, AppState>,
    id: String,
    note: String,
) -> Result<(), String> {
    let mut db = state.db.lock().unwrap();
    db.update_character_note(&id, &note)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_character(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let mut db = state.db.lock().unwrap();
    db.delete_character(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_dungeon_defs(state: State<'_, AppState>) -> Result<Vec<DungeonDef>, String> {
    let db = state.db.lock().unwrap();
    db.list_dungeon_defs().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_clear_records(state: State<'_, AppState>) -> Result<Vec<ClearRecord>, String> {
    let db = state.db.lock().unwrap();
    db.list_clear_records().map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn update_clear_record(
    state: State<'_, AppState>,
    character_id: String,
    dungeon_id: String,
    current_clears: u32,
) -> Result<(), String> {
    let mut db = state.db.lock().unwrap();
    let dungeons = db.list_dungeon_defs().map_err(|e| e.to_string())?;
    let max_clears = dungeons
        .iter()
        .find(|d| d.id == dungeon_id)
        .map(|d| d.max_clears)
        .unwrap_or(1);
    let week_start = chrono::Utc::now().naive_utc().date();
    db.update_clear_record(
        &character_id,
        &dungeon_id,
        current_clears,
        max_clears,
        week_start,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn manual_reset_all_cds(state: State<'_, AppState>) -> Result<(), String> {
    let mut db = state.db.lock().unwrap();
    db.reset_all_cds().map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn add_dungeon_def(
    state: State<'_, AppState>,
    name: String,
    short_name: String,
    max_clears: u32,
    reset_day: Option<u8>,
    reset_hour: Option<u8>,
    note: Option<String>,
) -> Result<(), String> {
    let mut db = state.db.lock().unwrap();
    let mut dungeon =
        crate::models::character::DungeonDef::new(&name, &short_name, vec![], max_clears);
    dungeon.reset_day = reset_day.unwrap_or(6).min(6);
    dungeon.reset_hour = reset_hour.unwrap_or(9).min(23);
    dungeon.note = note;
    let sort_order = db.list_dungeon_defs().map_err(|e| e.to_string())?.len() as i32;
    db.save_dungeon_def(&dungeon, sort_order)
        .map_err(|e| e.to_string())?;

    // Initialize clear records for all existing characters
    let chars = db.list_characters().map_err(|e| e.to_string())?;
    let week_start = chrono::Utc::now().naive_utc().date();
    for character in chars {
        db.update_clear_record(
            &character.id,
            &dungeon.id,
            0,
            dungeon.max_clears,
            week_start,
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub fn update_dungeon_def(
    state: State<'_, AppState>,
    id: String,
    name: String,
    short_name: String,
    max_clears: u32,
    reset_day: u8,
    reset_hour: u8,
    note: Option<String>,
) -> Result<(), String> {
    let mut db = state.db.lock().unwrap();
    let mut dungeons = db.list_dungeon_defs().map_err(|e| e.to_string())?;
    let pos = dungeons
        .iter()
        .position(|d| d.id == id)
        .ok_or("Dungeon not found")?;
    let mut dungeon = dungeons.remove(pos);
    dungeon.name = name;
    dungeon.short_name = short_name;
    dungeon.keywords = vec![dungeon.name.clone(), dungeon.short_name.clone()];
    dungeon.max_clears = max_clears.max(1);
    dungeon.reset_day = reset_day.min(6);
    dungeon.reset_hour = reset_hour.min(23);
    dungeon.note = note;
    db.save_dungeon_def(&dungeon, dungeon.sort_order)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_dungeon_def(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let mut db = state.db.lock().unwrap();
    db.delete_dungeon_def(&id).map_err(|e| e.to_string())
}
