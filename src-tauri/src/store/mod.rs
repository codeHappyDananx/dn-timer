use chrono::{DateTime, NaiveDate, Utc};
use rusqlite::{Connection, OptionalExtension};
use serde_json;
use tauri::{AppHandle, Manager};
use uuid::Uuid;

use crate::models::character::{Character, ClearRecord, DungeonDef};
use crate::models::config::AppConfig;
use crate::models::hotkey::HotkeyBindings;
use crate::models::preset::TimerPreset;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(app_handle: AppHandle) -> anyhow::Result<Self> {
        let app_dir = app_handle
            .path()
            .app_data_dir()
            .map_err(|e| anyhow::anyhow!("Failed to get app data dir: {}", e))?;
        std::fs::create_dir_all(&app_dir)?;
        let db_path = app_dir.join("data.db");
        let conn = Connection::open(&db_path)?;
        let db = Self { conn };
        db.run_migrations()?;
        let _ = db.dedup_clear_records();
        Ok(db)
    }

    fn run_migrations(&self) -> anyhow::Result<()> {
        let sql = include_str!("migrations/001_init.sql");
        self.conn.execute_batch(sql)?;
        let _ = self
            .conn
            .execute("ALTER TABLE characters ADD COLUMN class_key TEXT", []);
        let _ = self
            .conn
            .execute("ALTER TABLE characters ADD COLUMN note TEXT", []);
        let _ = self
            .conn
            .execute("ALTER TABLE dungeon_defs ADD COLUMN note TEXT", []);
        Ok(())
    }

    fn dedup_clear_records(&self) -> anyhow::Result<()> {
        self.conn.execute(
            "DELETE FROM clear_records WHERE id NOT IN (
                SELECT MAX(id) FROM clear_records GROUP BY character_id, dungeon_id
            )",
            [],
        )?;
        Ok(())
    }

    fn normalize_builtin_preset_names(&mut self) -> anyhow::Result<()> {
        let mut stmt = self.conn.prepare("SELECT id, data FROM presets")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        let mut updates: Vec<TimerPreset> = Vec::new();
        for row in rows {
            let (_, data) = row?;
            let Ok(mut preset) = serde_json::from_str::<TimerPreset>(&data) else {
                continue;
            };
            if !preset.is_builtin {
                continue;
            }

            let normalized_name = match preset.name.as_str() {
                "第二关·全屏毒" => Some("绿龙经典-第二关·全屏毒"),
                "第三关·石化诅咒" => Some("绿龙经典-第三关·石化诅咒"),
                "第四关·虫潮" => Some("绿龙经典-第四关·虫潮"),
                "卡拉翰·三机制" => Some("绿龙经典-第五关·卡拉翰"),
                _ => None,
            };

            if let Some(name) = normalized_name {
                preset.name = name.to_string();
                preset.updated_at = Utc::now();
                updates.push(preset);
            }
        }
        drop(stmt);

        for preset in updates {
            let data = serde_json::to_string(&preset)?;
            self.conn.execute(
                "UPDATE OR IGNORE presets
                 SET name = ?1, data = ?2, is_builtin = ?3, updated_at = ?4
                 WHERE id = ?5",
                [
                    &preset.name,
                    &data,
                    &preset.is_builtin.to_string(),
                    &preset.updated_at.to_rfc3339(),
                    &preset.id,
                ],
            )?;
        }

        Ok(())
    }

    pub fn seed_builtin_presets(&mut self) -> anyhow::Result<()> {
        self.normalize_builtin_preset_names()?;

        if self
            .get_config("builtin_presets_seeded")?
            .as_deref()
            .and_then(|v| v.parse::<bool>().ok())
            .unwrap_or(false)
        {
            return Ok(());
        }

        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM presets", [], |row| row.get(0))?;
        if count > 0 {
            self.set_config("builtin_presets_seeded", "true")?;
            return Ok(());
        }

        let presets = vec![
            TimerPreset {
                id: Uuid::new_v4().to_string(),
                name: "绿龙经典-第二关·全屏毒".to_string(),
                desc: Some("45秒循环：集合→找旋风台子躲避全屏毒".to_string()),
                hint: Some("BUFF剩30秒时退到中间台子集合".to_string()),
                note: None,
                preset_type: crate::models::preset::PresetType::Single {
                    first: 45,
                    loop_interval: 45,
                    followups: vec![],
                    warn_seconds: 10,
                    warn_text: Some("准备集合找旋风！".to_string()),
                    hotkey: None,
                    bar_color: None,
                    text_color: None,
                },
                created_at: Utc::now(),
                updated_at: Utc::now(),
                is_builtin: true,
            },
            TimerPreset {
                id: Uuid::new_v4().to_string(),
                name: "绿龙经典-第三关·石化诅咒".to_string(),
                desc: Some("首波60秒，之后45秒循环".to_string()),
                hint: Some("吹飞后按S前进，每人分散躲不同石像".to_string()),
                note: None,
                preset_type: crate::models::preset::PresetType::Single {
                    first: 60,
                    loop_interval: 45,
                    followups: vec![],
                    warn_seconds: 8,
                    warn_text: Some("准备分散躲石像！".to_string()),
                    hotkey: None,
                    bar_color: None,
                    text_color: None,
                },
                created_at: Utc::now(),
                updated_at: Utc::now(),
                is_builtin: true,
            },
            TimerPreset {
                id: Uuid::new_v4().to_string(),
                name: "绿龙经典-第四关·虫潮".to_string(),
                desc: Some("45秒循环：集合→跳蘑菇躲避全屏虫群".to_string()),
                hint: Some("BUFF剩30秒时向中间集中".to_string()),
                note: None,
                preset_type: crate::models::preset::PresetType::Single {
                    first: 45,
                    loop_interval: 45,
                    followups: vec![],
                    warn_seconds: 10,
                    warn_text: Some("准备上蘑菇！".to_string()),
                    hotkey: None,
                    bar_color: None,
                    text_color: None,
                },
                created_at: Utc::now(),
                updated_at: Utc::now(),
                is_builtin: true,
            },
            TimerPreset {
                id: Uuid::new_v4().to_string(),
                name: "绿龙经典-第五关·卡拉翰".to_string(),
                desc: Some("三个90秒独立计时：元气蛋/粪球/黑洞".to_string()),
                hint: Some("给每个槽位绑一个快捷键".to_string()),
                note: None,
                preset_type: crate::models::preset::PresetType::Multi {
                    slots: vec![
                        crate::models::preset::TimerSlotDef {
                            id: Uuid::new_v4().to_string(),
                            name: "元气蛋".to_string(),
                            first: 90,
                            loop_interval: 90,
                            warn_seconds: 10,
                            warn_text: Some("元气蛋来了！".to_string()),
                            hotkey: Some("F1".to_string()),
                            bar_color: None,
                            text_color: None,
                        },
                        crate::models::preset::TimerSlotDef {
                            id: Uuid::new_v4().to_string(),
                            name: "粪球".to_string(),
                            first: 45,
                            loop_interval: 45,
                            warn_seconds: 5,
                            warn_text: Some("粪球来了！".to_string()),
                            hotkey: Some("F2".to_string()),
                            bar_color: None,
                            text_color: None,
                        },
                        crate::models::preset::TimerSlotDef {
                            id: Uuid::new_v4().to_string(),
                            name: "黑洞".to_string(),
                            first: 60,
                            loop_interval: 60,
                            warn_seconds: 5,
                            warn_text: Some("黑洞来了！".to_string()),
                            hotkey: Some("F3".to_string()),
                            bar_color: None,
                            text_color: None,
                        },
                    ],
                    sequential: false,
                },
                created_at: Utc::now(),
                updated_at: Utc::now(),
                is_builtin: true,
            },
        ];

        for preset in presets {
            self.save_preset(&preset)?;
        }
        self.set_config("builtin_presets_seeded", "true")?;

        Ok(())
    }

    pub fn seed_builtin_dungeon_defs(&mut self) -> anyhow::Result<()> {
        let desired = vec![
            (
                "海龙(硬核)",
                "海龙(硬核)",
                vec!["海龙", "硬核"],
                vec!["海龙巢穴（硬核模式）", "海龙", "海龙硬核"],
            ),
            (
                "大主教(地狱)",
                "大主教(地狱)",
                vec!["大主教", "地狱"],
                vec!["大主教巢穴（地狱模式）", "大主教", "大主教地狱"],
            ),
            (
                "大主教（月卡）",
                "大主教（月卡）",
                vec!["大主教", "月卡"],
                vec!["大主教月卡"],
            ),
            (
                "绿龙（小绿）",
                "绿龙（小绿）",
                vec!["绿龙", "小绿"],
                vec!["绿龙巢穴", "绿龙", "小绿"],
            ),
            (
                "绿龙（经典）",
                "绿龙（经典）",
                vec!["绿龙", "经典"],
                vec!["绿龙巢穴（经典模式）", "绿龙经典"],
            ),
            (
                "绿龙（硬核）",
                "绿龙（硬核）",
                vec!["绿龙", "硬核"],
                vec!["绿龙硬核"],
            ),
            (
                "绿龙（月卡）",
                "绿龙（月卡）",
                vec!["绿龙", "月卡"],
                vec!["绿龙月卡"],
            ),
        ];

        let existing_count: i64 =
            self.conn
                .query_row("SELECT COUNT(*) FROM dungeon_defs", [], |row| row.get(0))?;
        if existing_count > 0 {
            return Ok(());
        }

        let chars = self.list_characters()?;
        let week_start = Utc::now().naive_utc().date();

        for (sort_order, (name, short_name, keywords, _aliases)) in desired.into_iter().enumerate() {
            let dungeon = DungeonDef::new(name, short_name, keywords, 1);
            self.save_dungeon_def(&dungeon, sort_order as i32)?;

            for character in &chars {
                let existing_record: Option<i64> = self
                    .conn
                    .query_row(
                        "SELECT id FROM clear_records WHERE character_id = ?1 AND dungeon_id = ?2",
                        [&character.id, &dungeon.id],
                        |row| row.get(0),
                    )
                    .optional()?;
                if existing_record.is_none() {
                    self.update_clear_record(
                        &character.id,
                        &dungeon.id,
                        0,
                        dungeon.max_clears,
                        week_start,
                    )?;
                }
            }
        }

        Ok(())
    }

    // Preset CRUD
    pub fn save_preset(&mut self, preset: &TimerPreset) -> anyhow::Result<()> {
        let data = serde_json::to_string(preset)?;
        self.conn.execute(
            "INSERT OR REPLACE INTO presets (id, name, data, is_builtin, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            [
                &preset.id,
                &preset.name,
                &data,
                &preset.is_builtin.to_string(),
                &preset.created_at.to_rfc3339(),
                &preset.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn get_preset(&self, id: &str) -> anyhow::Result<Option<TimerPreset>> {
        let mut stmt = self
            .conn
            .prepare("SELECT data FROM presets WHERE id = ?1")?;
        let data: Option<String> = stmt.query_row([id], |row| row.get(0)).optional()?;
        match data {
            Some(d) => Ok(Some(serde_json::from_str(&d)?)),
            None => Ok(None),
        }
    }

    pub fn list_presets(&self) -> anyhow::Result<Vec<TimerPreset>> {
        let mut stmt = self
            .conn
            .prepare("SELECT data FROM presets ORDER BY updated_at DESC")?;
        let rows = stmt.query_map([], |row| {
            let data: String = row.get(0)?;
            Ok(serde_json::from_str::<TimerPreset>(&data).unwrap())
        })?;
        let mut presets = Vec::new();
        for row in rows {
            presets.push(row?);
        }
        Ok(presets)
    }

    pub fn delete_preset(&mut self, id: &str) -> anyhow::Result<()> {
        self.conn
            .execute("DELETE FROM presets WHERE id = ?1", [id])?;
        Ok(())
    }

    // Character CRUD
    pub fn add_character(&mut self, character: &Character) -> anyhow::Result<()> {
        self.conn.execute(
            "INSERT INTO characters (id, name, server, class_key, note, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            [
                &character.id,
                &character.name,
                character.server.as_deref().unwrap_or(""),
                character.class_key.as_deref().unwrap_or(""),
                character.note.as_deref().unwrap_or(""),
                &character.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn list_characters(&self) -> anyhow::Result<Vec<Character>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, server, class_key, note, created_at FROM characters ORDER BY created_at",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Character {
                id: row.get(0)?,
                name: row.get(1)?,
                server: row.get::<_, Option<String>>(2)?,
                class_key: row.get::<_, Option<String>>(3)?,
                note: row.get::<_, Option<String>>(4)?,
                created_at: row
                    .get::<_, String>(5)?
                    .parse::<DateTime<Utc>>()
                    .unwrap_or_else(|_| Utc::now()),
            })
        })?;
        let mut chars = Vec::new();
        for row in rows {
            chars.push(row?);
        }
        Ok(chars)
    }

    pub fn update_character_class(
        &mut self,
        id: &str,
        class_key: Option<&str>,
    ) -> anyhow::Result<()> {
        self.conn.execute(
            "UPDATE characters SET class_key = ?1 WHERE id = ?2",
            [class_key.unwrap_or(""), id],
        )?;
        Ok(())
    }

    pub fn update_character_note(&mut self, id: &str, note: &str) -> anyhow::Result<()> {
        self.conn
            .execute("UPDATE characters SET note = ?1 WHERE id = ?2", [note, id])?;
        Ok(())
    }

    pub fn delete_character(&mut self, id: &str) -> anyhow::Result<()> {
        self.conn
            .execute("DELETE FROM characters WHERE id = ?1", [id])?;
        self.conn
            .execute("DELETE FROM clear_records WHERE character_id = ?1", [id])?;
        Ok(())
    }

    // Dungeon Def
    pub fn save_dungeon_def(
        &mut self,
        dungeon: &DungeonDef,
        sort_order: i32,
    ) -> anyhow::Result<()> {
        let keywords = serde_json::to_string(&dungeon.keywords)?;
        self.conn.execute(
            "INSERT OR REPLACE INTO dungeon_defs (id, name, short_name, keywords, icon, max_clears, reset_day, reset_hour, note, sort_order)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            [
                &dungeon.id,
                &dungeon.name,
                &dungeon.short_name,
                &keywords,
                dungeon.icon.as_deref().unwrap_or(""),
                &dungeon.max_clears.to_string(),
                &dungeon.reset_day.to_string(),
                &dungeon.reset_hour.to_string(),
                dungeon.note.as_deref().unwrap_or(""),
                &sort_order.to_string(),
            ],
        )?;
        Ok(())
    }

    pub fn list_dungeon_defs(&self) -> anyhow::Result<Vec<DungeonDef>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, short_name, keywords, icon, max_clears, reset_day, reset_hour, note, sort_order FROM dungeon_defs ORDER BY sort_order"
        )?;
        let rows = stmt.query_map([], |row| {
            let keywords: String = row.get(3)?;
            Ok(DungeonDef {
                id: row.get(0)?,
                name: row.get(1)?,
                short_name: row.get(2)?,
                keywords: serde_json::from_str(&keywords).unwrap_or_default(),
                icon: row.get::<_, Option<String>>(4)?,
                max_clears: row.get::<_, i64>(5)? as u32,
                reset_day: row.get::<_, i64>(6)? as u8,
                reset_hour: row.get::<_, i64>(7)? as u8,
                note: row.get::<_, Option<String>>(8)?,
                sort_order: row.get::<_, i64>(9)? as i32,
            })
        })?;
        let mut dungeons = Vec::new();
        for row in rows {
            dungeons.push(row?);
        }
        Ok(dungeons)
    }

    pub fn delete_dungeon_def(&mut self, id: &str) -> anyhow::Result<()> {
        self.conn
            .execute("DELETE FROM dungeon_defs WHERE id = ?1", [id])?;
        self.conn
            .execute("DELETE FROM clear_records WHERE dungeon_id = ?1", [id])?;
        Ok(())
    }

    // Clear Records
    pub fn update_clear_record(
        &mut self,
        character_id: &str,
        dungeon_id: &str,
        current_clears: u32,
        max_clears: u32,
        week_start: NaiveDate,
    ) -> anyhow::Result<()> {
        let existing: Option<i64> = self
            .conn
            .query_row(
                "SELECT id FROM clear_records WHERE character_id = ?1 AND dungeon_id = ?2",
                [character_id, dungeon_id],
                |row| row.get(0),
            )
            .ok();
        let now = Utc::now().to_rfc3339();
        if let Some(id) = existing {
            self.conn.execute(
                "UPDATE clear_records SET current_clears = ?1, max_clears = ?2, week_start = ?3, last_updated = ?4 WHERE id = ?5",
                [
                    &current_clears.to_string(),
                    &max_clears.to_string(),
                    &week_start.to_string(),
                    &now,
                    &id.to_string(),
                ],
            )?;
        } else {
            self.conn.execute(
                "INSERT INTO clear_records (character_id, dungeon_id, current_clears, max_clears, week_start, last_updated)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                [
                    character_id,
                    dungeon_id,
                    &current_clears.to_string(),
                    &max_clears.to_string(),
                    &week_start.to_string(),
                    &now,
                ],
            )?;
        }
        Ok(())
    }

    pub fn list_clear_records(&self) -> anyhow::Result<Vec<ClearRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT character_id, dungeon_id, current_clears, max_clears, week_start, last_updated FROM clear_records ORDER BY last_updated DESC"
        )?;
        let rows = stmt.query_map([], |row| {
            let week_str: String = row.get(4)?;
            Ok(ClearRecord {
                character_id: row.get(0)?,
                dungeon_id: row.get(1)?,
                current_clears: row.get::<_, i64>(2)? as u32,
                max_clears: row.get::<_, i64>(3)? as u32,
                week_start: NaiveDate::parse_from_str(&week_str, "%Y-%m-%d")
                    .unwrap_or_else(|_| Utc::now().naive_utc().date()),
                last_updated: row
                    .get::<_, String>(5)?
                    .parse::<DateTime<Utc>>()
                    .unwrap_or_else(|_| Utc::now()),
            })
        })?;
        let mut records = Vec::new();
        for row in rows {
            records.push(row?);
        }
        Ok(records)
    }

    pub fn reset_all_cds(&mut self) -> anyhow::Result<()> {
        let now = Utc::now().naive_utc().date();
        self.conn.execute(
            "UPDATE clear_records SET current_clears = 0, last_updated = ?1",
            [&now.to_string()],
        )?;
        Ok(())
    }

    pub fn reset_cds_for_dungeons(&mut self, dungeon_ids: &[String]) -> anyhow::Result<()> {
        if dungeon_ids.is_empty() {
            return Ok(());
        }
        let now = Utc::now().naive_utc().date().to_string();
        let tx = self.conn.transaction()?;
        for dungeon_id in dungeon_ids {
            tx.execute(
                "UPDATE clear_records SET current_clears = 0, last_updated = ?1 WHERE dungeon_id = ?2",
                [&now, dungeon_id],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    // Config
    pub fn get_config(&self, key: &str) -> anyhow::Result<Option<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT value FROM app_config WHERE key = ?1")?;
        let value: Option<String> = stmt.query_row([key], |row| row.get(0)).optional()?;
        Ok(value)
    }

    pub fn set_config(&mut self, key: &str, value: &str) -> anyhow::Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO app_config (key, value) VALUES (?1, ?2)",
            [key, value],
        )?;
        Ok(())
    }

    pub fn get_warn_config(&self) -> anyhow::Result<AppConfig> {
        let mut config = AppConfig::default();
        if let Some(v) = self.get_config("warn_seconds")? {
            config.warn_seconds = v.parse().unwrap_or(10);
        }
        if let Some(v) = self.get_config("warn_sound_enabled")? {
            config.warn_sound_enabled = v.parse().unwrap_or(true);
        }
        if let Some(v) = self.get_config("warn_sound_key")? {
            config.warn_sound_key = v;
        }
        if let Some(v) = self.get_config("warn_sound_repeat")? {
            config.warn_sound_repeat = v;
        }
        Ok(config)
    }

    pub fn set_warn_config(&mut self, config: &AppConfig) -> anyhow::Result<()> {
        self.set_config("warn_seconds", &config.warn_seconds.to_string())?;
        self.set_config("warn_sound_enabled", &config.warn_sound_enabled.to_string())?;
        self.set_config("warn_sound_key", &config.warn_sound_key)?;
        self.set_config("warn_sound_repeat", &config.warn_sound_repeat)?;
        Ok(())
    }

    // Hotkey bindings
    pub fn get_hotkey_bindings(&self) -> anyhow::Result<HotkeyBindings> {
        let mut stmt = self
            .conn
            .prepare("SELECT action, spec FROM hotkey_bindings")?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?.unwrap_or_default(),
            ))
        })?;
        let mut bindings = HotkeyBindings::new();
        for row in rows {
            let (action, spec) = row?;
            bindings.insert(action, spec);
        }
        Ok(bindings)
    }

    pub fn set_hotkey_bindings(&mut self, bindings: &HotkeyBindings) -> anyhow::Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute("DELETE FROM hotkey_bindings", [])?;
        for (action, spec) in bindings {
            tx.execute(
                "INSERT INTO hotkey_bindings (action, spec) VALUES (?1, ?2)",
                [action, spec],
            )?;
        }
        tx.commit()?;
        Ok(())
    }
}
