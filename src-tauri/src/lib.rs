pub mod commands;
pub mod db;
pub mod error;
pub mod models;
pub mod stopwatch;

use std::sync::Mutex;

use rusqlite::Connection;
use tauri::Manager;

use crate::error::AppError;

pub struct AppState {
    pub db: Mutex<Connection>,
}

impl AppState {
    pub fn lock_db(&self) -> Result<std::sync::MutexGuard<'_, Connection>, AppError> {
        self.db.lock().map_err(|_| AppError::StateLockPoisoned)
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            let db_path = app_data_dir.join("multi-stopwatch.db");
            let mut conn = db::init_db(&db_path)?;

            if let Ok(now) = stopwatch::now_ms()
                && let Err(err) = db::recover_crashed_running(&mut conn, now)
            {
                eprintln!("failed to recover crashed stopwatches: {err}");
            }

            app.manage(AppState {
                db: Mutex::new(conn),
            });
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event
                && let Some(state) = window.try_state::<AppState>()
                && let (Ok(mut conn), Ok(now)) = (state.lock_db(), stopwatch::now_ms())
                && let Err(err) = db::pause_all_running(&mut conn, now)
            {
                eprintln!("failed to pause running stopwatches on close: {err}");
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_stopwatches,
            commands::create_stopwatch,
            commands::rename_stopwatch,
            commands::start_stopwatch,
            commands::pause_stopwatch,
            commands::reset_stopwatch,
            commands::delete_stopwatch,
        ]);

    if let Err(err) = builder.run(tauri::generate_context!()) {
        eprintln!("error while running tauri application: {err}");
    }
}
