use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

use crate::store::Database;

pub fn start_scheduler(db: Arc<std::sync::Mutex<Database>>, app_handle: AppHandle) {
    thread::spawn(move || {
        use chrono::{Datelike, Timelike};

        let mut last_reset_key = String::new();
        loop {
            let now = chrono::Local::now();
            if now.minute() == 0 {
                let reset_day = now.weekday().num_days_from_sunday() as u8;
                let reset_hour = now.hour() as u8;
                let key = format!("{}-{}-{}", now.date_naive(), reset_day, reset_hour);

                if key != last_reset_key {
                    let dungeon_ids = {
                        let db_guard = db.lock().unwrap();
                        match db_guard.list_dungeon_defs() {
                            Ok(dungeons) => dungeons
                                .into_iter()
                                .filter(|d| d.reset_day == reset_day && d.reset_hour == reset_hour)
                                .map(|d| d.id)
                                .collect::<Vec<_>>(),
                            Err(e) => {
                                eprintln!("Failed to read dungeon_defs for scheduler: {}", e);
                                Vec::new()
                            }
                        }
                    };

                    if !dungeon_ids.is_empty() {
                        if let Ok(mut db_guard) = db.lock() {
                            let _ = db_guard.reset_cds_for_dungeons(&dungeon_ids);
                        }
                        let _ = app_handle.emit("cd:weekly_reset", ());
                    }
                    last_reset_key = key;
                }
            }

            thread::sleep(Duration::from_secs(30));
        }
    });
}
