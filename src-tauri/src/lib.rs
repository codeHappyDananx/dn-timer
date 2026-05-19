mod commands;
mod core;
mod models;
mod ocr;
mod overlay;
mod platform;
mod store;
mod utils;

use std::sync::{Arc, Mutex};
use tauri::Manager;

pub struct AppState {
    pub db: Arc<Mutex<store::Database>>,
    pub timer_engine: Arc<Mutex<core::TimerEngine>>,
    pub hotkey_manager: Arc<Mutex<platform::HotkeyManager>>,
    pub current_preset_id: Arc<Mutex<Option<String>>>,
    pub current_slots: Arc<Mutex<Vec<models::preset::TimerSlotDef>>>,
}

// Safety: rusqlite::Connection is !Sync but we only access it through Mutex which provides Sync
// because Mutex<T> is Sync when T: Send. This is safe as long as we never leak &T references.
unsafe impl Sync for AppState {}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                let app = window.app_handle();
                let state = app.state::<AppState>();
                let db = state.db.clone();
                let _ = platform::window::save_window_position(&app, &db);
            }
        })
        .setup(|app| {
            let app_handle = app.handle().clone();

            // Initialize database
            let db = store::Database::new(app_handle.clone())?;
            let db = Arc::new(Mutex::new(db));

            // Restore window flags
            {
                let db_guard = db.lock().unwrap();
                let always_on_top = db_guard
                    .get_config("always_on_top")
                    .ok()
                    .flatten()
                    .and_then(|v| v.parse::<bool>().ok())
                    .unwrap_or(false);
                if let Some(window) = app_handle.get_webview_window("main") {
                    let _ = window.set_always_on_top(always_on_top);
                    let _ = window.set_maximizable(false);
                    let _ = window.set_shadow(false);
                }
            }

            // Seed built-in presets and dungeon defs on first run
            {
                let mut db_guard = db.lock().unwrap();
                db_guard.seed_builtin_presets()?;
                db_guard.seed_builtin_dungeon_defs()?;
            }

            // Initialize timer engine
            let timer_engine = core::TimerEngine::new(app_handle.clone());
            let timer_engine = Arc::new(Mutex::new(timer_engine));

            // Load warn_config into engine
            {
                let db_guard = db.lock().unwrap();
                if let Ok(config) = db_guard.get_warn_config() {
                    let engine = timer_engine.lock().unwrap();
                    engine.send(core::EngineCommand::UpdateWarnConfig(core::WarnConfig {
                        sound_enabled: config.warn_sound_enabled,
                        sound_key: config.warn_sound_key.clone(),
                        sound_repeat: config.warn_sound_repeat.clone(),
                    }));
                }
            }

            // Initialize hotkey manager
            let mut hotkey_manager = platform::HotkeyManager::new(app_handle.clone());
            // Load saved bindings from DB
            if let Ok(bindings) = db.lock().unwrap().get_hotkey_bindings() {
                hotkey_manager.update_bindings(bindings);
            }
            hotkey_manager.start();
            let hotkey_manager = Arc::new(Mutex::new(hotkey_manager));

            // Start scheduler for weekly resets
            core::start_scheduler(db.clone(), app_handle.clone());

            // Create system tray
            let _ = platform::create_tray(&app_handle);

            // Start dock edge watcher
            platform::window::start_dock_watcher(app_handle.clone());

            // Phase 5: OCR Engine（预留初始化）
            // let ocr_engine = ocr::OcrEngine::new()?;
            // app.manage(Arc::new(Mutex::new(ocr_engine)));

            // Phase 5: Overlay Manager（预留初始化）
            // overlay::OverlayManager::create_overlay_window(&app_handle)?;

            app.manage(AppState {
                db,
                timer_engine,
                hotkey_manager,
                current_preset_id: Arc::new(Mutex::new(None)),
                current_slots: Arc::new(Mutex::new(Vec::new())),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::timer::start_timer,
            commands::timer::pause_timer,
            commands::timer::reset_timer,
            commands::timer::get_timer_snapshot,
            commands::timer::start_slot,
            commands::timer::pause_slot,
            commands::timer::reset_slot,
            commands::timer::toggle_slot,
            commands::timer::trigger_hotkey,
            commands::preset::list_presets,
            commands::preset::create_preset,
            commands::preset::update_preset,
            commands::preset::delete_preset,
            commands::preset::select_preset,
            commands::preset::next_preset,
            commands::preset::update_slot_hotkey,
            commands::preset::update_slot_config,
            commands::character::list_characters,
            commands::character::add_character,
            commands::character::update_character_class,
            commands::character::update_character_note,
            commands::character::delete_character,
            commands::character::list_dungeon_defs,
            commands::character::list_clear_records,
            commands::character::update_clear_record,
            commands::character::manual_reset_all_cds,
            commands::character::add_dungeon_def,
            commands::character::update_dungeon_def,
            commands::character::delete_dungeon_def,
            commands::hotkey::get_hotkey_bindings,
            commands::hotkey::set_hotkey_bindings,
            commands::config::get_warn_config,
            commands::config::set_warn_config,
            commands::window::close_window,
            commands::window::minimize_window,
            commands::window::show_window,
            commands::window::frontend_ready,
            commands::window::set_always_on_top,
            commands::window::get_always_on_top,
            commands::window::get_window_position,
            commands::window::resize_window,
            commands::window::begin_window_drag,
            commands::window::dock_window,
            commands::window::undock_window,
            commands::window::is_docked_window,
            commands::window::enter_simple_mode,
            commands::window::exit_simple_mode,
            commands::window::set_simple_mode,
            commands::window::get_simple_mode,
            commands::window::set_simple_mode_scale,
            commands::window::get_simple_mode_scale,
            commands::window::set_startup,
            commands::window::is_startup_enabled,
            commands::overlay::toggle_overlay,
            commands::overlay::set_overlay_click_through,
            commands::ocr::recognize_cd,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
