-- 模板表
CREATE TABLE IF NOT EXISTS presets (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    data TEXT NOT NULL,
    is_builtin INTEGER DEFAULT 0,
    created_at TEXT,
    updated_at TEXT
);

-- 角色表
CREATE TABLE IF NOT EXISTS characters (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    server TEXT,
    class_key TEXT,
    note TEXT,
    created_at TEXT
);

-- 巢穴定义表（完全可配置）
CREATE TABLE IF NOT EXISTS dungeon_defs (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    short_name TEXT NOT NULL,
    keywords TEXT NOT NULL,
    icon TEXT,
    max_clears INTEGER DEFAULT 1,
    reset_day INTEGER DEFAULT 6,
    reset_hour INTEGER DEFAULT 9,
    note TEXT,
    sort_order INTEGER DEFAULT 0
);

-- 通关记录表（按周分区）
CREATE TABLE IF NOT EXISTS clear_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    character_id TEXT NOT NULL,
    dungeon_id TEXT NOT NULL,
    current_clears INTEGER DEFAULT 0,
    max_clears INTEGER DEFAULT 1,
    week_start TEXT NOT NULL,
    last_updated TEXT,
    UNIQUE(character_id, dungeon_id, week_start)
);

-- 应用配置表
CREATE TABLE IF NOT EXISTS app_config (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- 快捷键配置表
CREATE TABLE IF NOT EXISTS hotkey_bindings (
    action TEXT PRIMARY KEY,
    spec TEXT
);
