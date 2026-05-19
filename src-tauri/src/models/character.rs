#![allow(dead_code)]

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    pub id: String,
    pub name: String,
    pub server: Option<String>,
    pub class_key: Option<String>,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl Character {
    pub fn new(name: &str, server: Option<&str>, class_key: Option<&str>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            server: server.map(|s| s.to_string()),
            class_key: class_key.map(|s| s.to_string()),
            note: None,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DungeonDef {
    pub id: String,
    pub name: String,
    pub short_name: String,
    pub keywords: Vec<String>,
    pub icon: Option<String>,
    pub max_clears: u32,
    pub reset_day: u8, // 0=Sunday, 6=Saturday
    pub reset_hour: u8,
    pub note: Option<String>,
    pub sort_order: i32,
}

impl DungeonDef {
    pub fn new(name: &str, short_name: &str, keywords: Vec<&str>, max_clears: u32) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            short_name: short_name.to_string(),
            keywords: keywords.iter().map(|s| s.to_string()).collect(),
            icon: None,
            max_clears,
            reset_day: 6, // Saturday
            reset_hour: 9,
            note: None,
            sort_order: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClearRecord {
    pub character_id: String,
    pub dungeon_id: String,
    pub current_clears: u32,
    pub max_clears: u32,
    pub week_start: NaiveDate,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterCDPanel {
    pub character: Character,
    pub dungeon_statuses: Vec<DungeonStatusItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DungeonStatusItem {
    pub dungeon: DungeonDef,
    pub current: u32,
    pub max: u32,
    pub status: ClearStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClearStatus {
    Available,
    Exhausted,
    Unknown,
}
