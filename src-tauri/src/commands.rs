use tauri::State;

use crate::AppState;
use crate::db;
use crate::error::AppError;
use crate::models::{Stopwatch, StopwatchDto};
use crate::stopwatch;

/// Helper internal untuk mengambil seluruh daftar stopwatch sebagai DTO.
pub fn list_stopwatches_inner(state: &AppState) -> Result<Vec<StopwatchDto>, AppError> {
    let conn = state.lock_db()?;
    let items = db::list_stopwatches(&conn)?;
    Ok(items.iter().map(Stopwatch::to_dto).collect())
}

/// Helper internal untuk membuat stopwatch baru.
pub fn create_stopwatch_inner(
    state: &AppState,
    name: Option<&str>,
) -> Result<StopwatchDto, AppError> {
    let now = stopwatch::now_ms()?;
    let conn = state.lock_db()?;
    let sw = db::create_stopwatch(&conn, name, now)?;
    Ok(sw.to_dto())
}

/// Helper internal untuk mengubah nama stopwatch berdasarkan ID.
pub fn rename_stopwatch_inner(
    state: &AppState,
    id: i64,
    name: &str,
) -> Result<StopwatchDto, AppError> {
    let now = stopwatch::now_ms()?;
    let conn = state.lock_db()?;
    let sw = db::rename_stopwatch(&conn, id, name, now)?;
    Ok(sw.to_dto())
}

/// Helper internal untuk menjalankan (start/resume) stopwatch secara idempotent.
pub fn start_stopwatch_inner(state: &AppState, id: i64) -> Result<StopwatchDto, AppError> {
    let now = stopwatch::now_ms()?;
    let conn = state.lock_db()?;
    let mut sw = db::get_stopwatch(&conn, id)?;
    if sw.started_at_ms.is_none() {
        stopwatch::start(&mut sw, now);
        db::save_stopwatch_state(&conn, &sw)?;
    }
    Ok(sw.to_dto())
}

/// Helper internal untuk menjeda (pause) stopwatch secara idempotent.
pub fn pause_stopwatch_inner(state: &AppState, id: i64) -> Result<StopwatchDto, AppError> {
    let now = stopwatch::now_ms()?;
    let conn = state.lock_db()?;
    let mut sw = db::get_stopwatch(&conn, id)?;
    if sw.started_at_ms.is_some() {
        stopwatch::pause(&mut sw, now);
        db::save_stopwatch_state(&conn, &sw)?;
    }
    Ok(sw.to_dto())
}

/// Helper internal untuk mereset stopwatch kembali ke 00:00:00.0 (`Idle`).
pub fn reset_stopwatch_inner(state: &AppState, id: i64) -> Result<StopwatchDto, AppError> {
    let now = stopwatch::now_ms()?;
    let conn = state.lock_db()?;
    let mut sw = db::get_stopwatch(&conn, id)?;
    stopwatch::reset(&mut sw, now);
    db::save_stopwatch_state(&conn, &sw)?;
    Ok(sw.to_dto())
}

/// Helper internal untuk menghapus stopwatch berdasarkan ID.
pub fn delete_stopwatch_inner(state: &AppState, id: i64) -> Result<(), AppError> {
    let conn = state.lock_db()?;
    db::delete_stopwatch(&conn, id)
}

#[tauri::command]
pub fn list_stopwatches(state: State<'_, AppState>) -> Result<Vec<StopwatchDto>, String> {
    list_stopwatches_inner(&state).map_err(Into::into)
}

#[tauri::command]
pub fn create_stopwatch(
    state: State<'_, AppState>,
    name: Option<String>,
) -> Result<StopwatchDto, String> {
    create_stopwatch_inner(&state, name.as_deref()).map_err(Into::into)
}

#[tauri::command]
pub fn rename_stopwatch(
    state: State<'_, AppState>,
    id: i64,
    name: String,
) -> Result<StopwatchDto, String> {
    rename_stopwatch_inner(&state, id, &name).map_err(Into::into)
}

#[tauri::command]
pub fn start_stopwatch(state: State<'_, AppState>, id: i64) -> Result<StopwatchDto, String> {
    start_stopwatch_inner(&state, id).map_err(Into::into)
}

#[tauri::command]
pub fn pause_stopwatch(state: State<'_, AppState>, id: i64) -> Result<StopwatchDto, String> {
    pause_stopwatch_inner(&state, id).map_err(Into::into)
}

#[tauri::command]
pub fn reset_stopwatch(state: State<'_, AppState>, id: i64) -> Result<StopwatchDto, String> {
    reset_stopwatch_inner(&state, id).map_err(Into::into)
}

#[tauri::command]
pub fn delete_stopwatch(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    delete_stopwatch_inner(&state, id).map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;
    use crate::models::StopwatchStatus;

    fn make_test_state() -> (tempfile::TempDir, AppState) {
        let dir = tempfile::tempdir().expect("create temp dir");
        let db_path = dir.path().join("test-commands.db");
        let conn = db::init_db(&db_path).expect("init_db should succeed");
        (dir, AppState { db: Mutex::new(conn) })
    }

    #[test]
    fn test_commands_create_list_rename_and_delete_flow() {
        let (_dir, state) = make_test_state();

        let initial = list_stopwatches_inner(&state).expect("list initial");
        assert!(initial.is_empty());

        let sw1 = create_stopwatch_inner(&state, None).expect("create default");
        assert_eq!(sw1.name, "Stopwatch 1");
        assert_eq!(sw1.status, StopwatchStatus::Idle);
        assert_eq!(sw1.position, 1);

        let sw2 = create_stopwatch_inner(&state, Some("  Fokus Koding  ")).expect("create named");
        assert_eq!(sw2.name, "Fokus Koding");
        assert_eq!(sw2.position, 2);

        let list = list_stopwatches_inner(&state).expect("list after create");
        assert_eq!(list.len(), 2);

        let renamed =
            rename_stopwatch_inner(&state, sw1.id, "  Sprint Pagi  ").expect("rename sw1");
        assert_eq!(renamed.name, "Sprint Pagi");

        // Validasi nama kosong atau terlalu panjang harus ditolak
        let empty_err = rename_stopwatch_inner(&state, sw1.id, "   ");
        assert!(matches!(empty_err, Err(AppError::EmptyName)));

        let long_name = "a".repeat(51);
        let long_err = rename_stopwatch_inner(&state, sw1.id, &long_name);
        assert!(matches!(long_err, Err(AppError::NameTooLong(51))));

        delete_stopwatch_inner(&state, sw1.id).expect("delete sw1");
        let after_delete = list_stopwatches_inner(&state).expect("list after delete");
        assert_eq!(after_delete.len(), 1);
        assert_eq!(after_delete[0].id, sw2.id);

        let missing_delete = delete_stopwatch_inner(&state, sw1.id);
        assert!(matches!(missing_delete, Err(AppError::NotFound(id)) if id == sw1.id));
    }

    #[test]
    fn test_commands_start_pause_reset_and_idempotency() {
        let (_dir, state) = make_test_state();

        let created = create_stopwatch_inner(&state, Some("Timer Uji")).expect("create");
        assert_eq!(created.status, StopwatchStatus::Idle);
        assert_eq!(created.started_at_ms, None);

        let started = start_stopwatch_inner(&state, created.id).expect("start");
        assert_eq!(started.status, StopwatchStatus::Running);
        assert!(started.started_at_ms.is_some());

        // Start kedua kali harus idempotent dan mempertahankan started_at_ms pertama
        let started_again = start_stopwatch_inner(&state, created.id).expect("start idempotent");
        assert_eq!(started_again.started_at_ms, started.started_at_ms);

        let paused = pause_stopwatch_inner(&state, created.id).expect("pause");
        assert_eq!(paused.started_at_ms, None);
        assert!(paused.accumulated_ms >= 0);

        // Pause kedua kali harus idempotent
        let paused_again = pause_stopwatch_inner(&state, created.id).expect("pause idempotent");
        assert_eq!(paused_again.accumulated_ms, paused.accumulated_ms);
        assert_eq!(paused_again.started_at_ms, None);

        let reset = reset_stopwatch_inner(&state, created.id).expect("reset");
        assert_eq!(reset.status, StopwatchStatus::Idle);
        assert_eq!(reset.accumulated_ms, 0);
        assert_eq!(reset.started_at_ms, None);

        // ID yang tidak ada harus mengembalikan NotFound
        assert!(matches!(
            start_stopwatch_inner(&state, 9999),
            Err(AppError::NotFound(9999))
        ));
        assert!(matches!(
            pause_stopwatch_inner(&state, 9999),
            Err(AppError::NotFound(9999))
        ));
        assert!(matches!(
            reset_stopwatch_inner(&state, 9999),
            Err(AppError::NotFound(9999))
        ));
    }
}
