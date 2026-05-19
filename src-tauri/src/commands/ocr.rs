use tauri::State;

use crate::platform::screenshot;
use crate::AppState;

#[tauri::command]
pub async fn recognize_cd(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    // 1. 查找游戏窗口
    let hwnd = match screenshot::find_game_window() {
        Some(h) => h,
        None => {
            return Ok(serde_json::json!({
                "success": false,
                "error": "未找到游戏窗口，请先打开游戏"
            }));
        }
    };

    // 2. 截图并保存到临时文件
    let temp_path = std::env::temp_dir().join("dn_timer_screenshot.bmp");
    let image_data = match screenshot::capture_window(hwnd) {
        Ok(data) => data,
        Err(e) => {
            return Ok(serde_json::json!({
                "success": false,
                "error": format!("截图失败: {}", e)
            }));
        }
    };

    if let Err(e) = std::fs::write(&temp_path, image_data) {
        return Ok(serde_json::json!({
            "success": false,
            "error": format!("截图保存失败: {}", e)
        }));
    }

    // 3. 调用 OCR（延迟初始化）
    let ocr = match crate::ocr::OcrEngine::new() {
        Ok(engine) => engine,
        Err(e) => {
            return Ok(serde_json::json!({
                "success": false,
                "error": format!("OCR引擎启动失败: {}", e)
            }));
        }
    };

    let result = match ocr.recognize(temp_path.to_str(), None) {
        Ok(r) => r,
        Err(e) => {
            return Ok(serde_json::json!({
                "success": false,
                "error": format!("OCR识别失败: {}", e)
            }));
        }
    };

    // 4. 更新数据库
    if result.success {
        if let Some(ref char_name) = result.character_name {
            if let Ok(mut db) = state.db.lock() {
                let chars = db.list_characters().unwrap_or_default();
                let char_id = chars
                    .iter()
                    .find(|c| c.name == *char_name)
                    .map(|c| c.id.clone());

                let char_id = match char_id {
                    Some(id) => id,
                    None => {
                        let new_char =
                            crate::models::character::Character::new(char_name, None, None);
                        let id = new_char.id.clone();
                        if db.add_character(&new_char).is_ok() {
                            id
                        } else {
                            String::new()
                        }
                    }
                };

                if !char_id.is_empty() {
                    if let Ok(dungeons) = db.list_dungeon_defs() {
                        for item in &result.dungeon_cds {
                            if let Some(dungeon) = dungeons.iter().find(|d| {
                                d.name == item.name
                                    || d.short_name == item.name
                                    || d.keywords.iter().any(|k| *k == item.name)
                            }) {
                                let current_clears = parse_clears(&item.count);
                                let week_start = chrono::Utc::now().naive_utc().date();
                                let _ = db.update_clear_record(
                                    &char_id,
                                    &dungeon.id,
                                    current_clears,
                                    dungeon.max_clears,
                                    week_start,
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    serde_json::to_value(result).map_err(|e| e.to_string())
}

fn parse_clears(s: &str) -> u32 {
    s.split('/')
        .next()
        .and_then(|p| p.trim().parse::<u32>().ok())
        .unwrap_or(0)
}
