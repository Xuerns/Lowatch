pub mod commands;
pub mod db;
pub mod error;
pub mod focus;
pub mod models;
pub mod stopwatch;
pub mod timer;

use std::sync::{
    Arc, Condvar, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;

use rusqlite::Connection;
use tauri::Manager;

use crate::error::AppError;
use crate::focus::FocusState;

/// Interval penulisan `heartbeat_ms` ke SQLite selama ada stopwatch berjalan (sesuai PRD 5.3 / F-12).
pub const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(15);

pub struct AppState {
    pub db: Mutex<Connection>,
    pub focus: Mutex<FocusState>,
    shutdown_flag: Arc<AtomicBool>,
    shutdown_wake: Arc<(Mutex<bool>, Condvar)>,
}

impl AppState {
    pub fn new(conn: Connection) -> Self {
        Self {
            db: Mutex::new(conn),
            focus: Mutex::new(FocusState::default()),
            shutdown_flag: Arc::new(AtomicBool::new(false)),
            shutdown_wake: Arc::new((Mutex::new(false), Condvar::new())),
        }
    }

    pub fn lock_db(&self) -> Result<std::sync::MutexGuard<'_, Connection>, AppError> {
        self.db.lock().map_err(|_| AppError::StateLockPoisoned)
    }

    pub fn lock_focus(&self) -> Result<std::sync::MutexGuard<'_, FocusState>, AppError> {
        self.focus.lock().map_err(|_| AppError::StateLockPoisoned)
    }

    /// Memberi sinyal penghentian ke background thread heartbeat dan membangunkan `Condvar` secara instan.
    pub fn signal_shutdown(&self) {
        self.shutdown_flag.store(true, Ordering::SeqCst);
        let (lock, cvar) = &*self.shutdown_wake;
        if let Ok(mut shutting_down) = lock.lock() {
            *shutting_down = true;
        }
        cvar.notify_all();
    }
}

/// Menjalankan satu siklus pembaruan `heartbeat_ms` untuk seluruh stopwatch yang sedang berjalan.
pub fn tick_heartbeat(state: &AppState) -> Result<usize, AppError> {
    let now = stopwatch::now_ms()?;
    let conn = state.lock_db()?;
    db::update_running_heartbeats(&conn, now)
}

/// Menjeda seluruh stopwatch yang sedang berjalan saat aplikasi ditutup (`CloseRequested` / `RunEvent::Exit`)
/// dan menghentikan background thread heartbeat secara idempotent.
pub fn pause_all_on_exit(state: &AppState) -> Result<usize, AppError> {
    state.signal_shutdown();
    let now = stopwatch::now_ms()?;
    let mut conn = state.lock_db()?;
    db::pause_all_running(&mut conn, now)
}

/// Menunggu selama `interval` menggunakan `Condvar::wait_timeout`, atau berhenti seketika saat `shutdown_flag` bernilai `true`.
/// Mengembalikan `true` jika thread harus menjalankan `tick_heartbeat`, atau `false` jika harus berhenti.
fn wait_for_heartbeat_or_shutdown(
    shutdown_flag: &AtomicBool,
    shutdown_wake: &(Mutex<bool>, Condvar),
    interval: Duration,
) -> bool {
    if shutdown_flag.load(Ordering::SeqCst) {
        return false;
    }
    let (lock, cvar) = shutdown_wake;
    let Ok(guard) = lock.lock() else {
        return false;
    };
    if *guard || shutdown_flag.load(Ordering::SeqCst) {
        return false;
    }
    let Ok((guard_after, _timeout)) = cvar.wait_timeout(guard, interval) else {
        return false;
    };
    !*guard_after && !shutdown_flag.load(Ordering::SeqCst)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                if let Err(err) = window.unminimize() {
                    eprintln!("failed to unminimize main window: {err}");
                }
                if let Err(err) = window.show() {
                    eprintln!("failed to show main window: {err}");
                }
                if let Err(err) = window.set_focus() {
                    eprintln!("failed to focus main window: {err}");
                }
            }
        }))
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            let db_path = app_data_dir.join("multi-stopwatch.db");
            let mut conn = db::init_db(&db_path)?;

            if let Ok(now) = stopwatch::now_ms()
                && let Err(err) = db::recover_crashed_running(&mut conn, now)
            {
                eprintln!("failed to recover crashed stopwatches: {err}");
            }

            let state = AppState::new(conn);
            let shutdown_flag = Arc::clone(&state.shutdown_flag);
            let shutdown_wake = Arc::clone(&state.shutdown_wake);
            app.manage(state);

            let app_handle = app.handle().clone();
            if let Err(err) = std::thread::Builder::new()
                .name("lowatch-heartbeat".to_string())
                .spawn(move || {
                    while wait_for_heartbeat_or_shutdown(
                        &shutdown_flag,
                        &shutdown_wake,
                        HEARTBEAT_INTERVAL,
                    ) {
                        if let Some(managed_state) = app_handle.try_state::<AppState>()
                            && let Err(err) = tick_heartbeat(&managed_state)
                        {
                            eprintln!("failed to update running heartbeats: {err}");
                        }
                    }
                })
            {
                eprintln!("failed to spawn heartbeat thread: {err}");
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event
                && let Some(state) = window.try_state::<AppState>()
                && let Err(err) = pause_all_on_exit(&state)
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
            commands::enter_focus_mode,
            commands::exit_focus_mode,
            commands::list_timers,
            commands::create_timer,
            commands::rename_timer,
            commands::set_timer_duration,
            commands::start_timer,
            commands::pause_timer,
            commands::complete_timer,
            commands::reset_timer,
            commands::delete_timer,
        ]);

    let app = match builder.build(tauri::generate_context!()) {
        Ok(app) => app,
        Err(err) => {
            eprintln!("error while building tauri application: {err}");
            return;
        }
    };

    app.run(|app_handle, event| {
        if let tauri::RunEvent::Exit = event
            && let Some(state) = app_handle.try_state::<AppState>()
            && let Err(err) = pause_all_on_exit(&state)
        {
            eprintln!("failed to pause running stopwatches on exit: {err}");
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::StopwatchStatus;

    fn make_test_state() -> (tempfile::TempDir, std::path::PathBuf, AppState) {
        let dir = tempfile::tempdir().expect("create temp dir");
        let db_path = dir.path().join("test-lib.db");
        let conn = db::init_db(&db_path).expect("init_db should succeed");
        (dir, db_path, AppState::new(conn))
    }

    #[test]
    fn test_pause_all_on_exit_is_idempotent_across_close_and_exit_events() {
        let (_dir, _db_path, state) = make_test_state();

        let sw = commands::create_stopwatch_inner(&state, Some("Sprint Backend"))
            .expect("create stopwatch");
        let started =
            commands::start_stopwatch_inner(&state, sw.id).expect("start stopwatch");
        assert_eq!(started.status, StopwatchStatus::Running);
        let started_at = started.started_at_ms.expect("started_at_ms must be set");

        // Pastikan jam sistem maju minimal 1 ms agar pause menghasilkan delta > 0 (status Paused)
        while stopwatch::now_ms().expect("now_ms") <= started_at {
            std::thread::sleep(Duration::from_millis(1));
        }

        // Simulasi panggilan pertama pada WindowEvent::CloseRequested
        let first_paused = pause_all_on_exit(&state).expect("first pause_all_on_exit");
        assert_eq!(first_paused, 1);
        assert!(state.shutdown_flag.load(Ordering::SeqCst));

        let after_first = {
            let conn = state.lock_db().expect("lock db");
            db::get_stopwatch(&conn, sw.id).expect("get sw after first pause")
        };
        assert_eq!(after_first.status(), StopwatchStatus::Paused);
        assert_eq!(after_first.started_at_ms, None);
        assert_eq!(after_first.heartbeat_ms, None);

        // Simulasi panggilan kedua pada RunEvent::Exit beberapa milidetik kemudian
        let second_paused = pause_all_on_exit(&state).expect("second pause_all_on_exit");
        assert_eq!(second_paused, 0);

        let after_second = {
            let conn = state.lock_db().expect("lock db");
            db::get_stopwatch(&conn, sw.id).expect("get sw after second pause")
        };
        assert_eq!(after_second.status(), StopwatchStatus::Paused);
        assert_eq!(after_second.accumulated_ms, after_first.accumulated_ms);
        assert_eq!(after_second.started_at_ms, None);
    }

    #[test]
    fn test_tick_heartbeat_and_startup_crash_recovery_flow() {
        let (_dir, db_path, state) = make_test_state();

        let sw_id = {
            let conn = state.lock_db().expect("lock db");
            let mut sw = db::create_stopwatch(&conn, Some("Ketahanan Crash"), 1_000)
                .expect("create sw");
            stopwatch::start(&mut sw, 1_000);
            db::save_stopwatch_state(&conn, &sw).expect("save running state");
            sw.id
        };

        // Jalankan tick_heartbeat pada AppState dan pastikan heartbeat_ms diperbarui
        let updated_count = tick_heartbeat(&state).expect("tick_heartbeat should succeed");
        assert_eq!(updated_count, 1);

        let hb_after_tick = {
            let conn = state.lock_db().expect("lock db");
            let sw = db::get_stopwatch(&conn, sw_id).expect("get sw after tick");
            assert_eq!(sw.status(), StopwatchStatus::Running);
            let hb = sw.heartbeat_ms.expect("heartbeat_ms must be present");
            assert!(hb >= 1_000);
            hb
        };

        // Pastikan worker thread yang menunggu di wait_for_heartbeat_or_shutdown berhenti instan saat signal_shutdown dipanggil
        let shutdown_flag = Arc::clone(&state.shutdown_flag);
        let shutdown_wake = Arc::clone(&state.shutdown_wake);
        let worker = std::thread::spawn(move || {
            wait_for_heartbeat_or_shutdown(
                &shutdown_flag,
                &shutdown_wake,
                Duration::from_secs(60),
            )
        });
        state.signal_shutdown();
        let should_continue = worker.join().expect("worker thread should join cleanly");
        assert!(!should_continue);

        // Simulasikan pembukaan ulang aplikasi setelah crash dan jalankan recover_crashed_running
        drop(state);
        let mut reopened_conn = db::init_db(&db_path).expect("reopen db after simulated crash");
        let recovered_now = hb_after_tick.saturating_add(30_000);
        let recovered_count = db::recover_crashed_running(&mut reopened_conn, recovered_now)
            .expect("recover crashed running");
        assert_eq!(recovered_count, 1);

        let recovered_sw =
            db::get_stopwatch(&reopened_conn, sw_id).expect("get recovered stopwatch");
        assert_eq!(recovered_sw.status(), StopwatchStatus::Paused);
        assert_eq!(recovered_sw.started_at_ms, None);
        assert_eq!(recovered_sw.heartbeat_ms, None);
        assert_eq!(recovered_sw.accumulated_ms, hb_after_tick - 1_000);
    }
}
