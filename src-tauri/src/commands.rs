use tauri::State;

use crate::AppState;
use crate::alarm;
use crate::db;
use crate::error::AppError;
use crate::models::{Alarm, AlarmDto, Stopwatch, StopwatchDto, Timer, TimerDto};
use crate::stopwatch;
use crate::timer;

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
    {
        let conn = state.lock_db()?;
        db::delete_stopwatch(&conn, id)?;
    }
    let mut focus = state.lock_focus()?;
    if focus.focused_stopwatch_id == Some(id) {
        focus.focused_stopwatch_id = None;
    }
    Ok(())
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

#[tauri::command]
pub fn enter_focus_mode(
    window: tauri::WebviewWindow,
    state: State<'_, AppState>,
    id: i64,
) -> Result<(), String> {
    crate::focus::enter_focus_mode(&window, &state, id).map_err(Into::into)
}

#[tauri::command]
pub fn exit_focus_mode(
    window: tauri::WebviewWindow,
    state: State<'_, AppState>,
) -> Result<(), String> {
    crate::focus::exit_focus_mode(&window, &state).map_err(Into::into)
}

/// Helper internal untuk mengambil seluruh daftar timer sebagai DTO.
pub fn list_timers_inner(state: &AppState) -> Result<Vec<TimerDto>, AppError> {
    let conn = state.lock_db()?;
    let items = db::list_timers(&conn)?;
    Ok(items.iter().map(Timer::to_dto).collect())
}

/// Helper internal untuk membuat timer baru.
pub fn create_timer_inner(
    state: &AppState,
    name: Option<&str>,
    duration_ms: Option<i64>,
) -> Result<TimerDto, AppError> {
    let now = stopwatch::now_ms()?;
    let conn = state.lock_db()?;
    let t = db::create_timer(&conn, name, duration_ms, now)?;
    Ok(t.to_dto())
}

/// Helper internal untuk mengubah nama timer berdasarkan ID.
pub fn rename_timer_inner(state: &AppState, id: i64, name: &str) -> Result<TimerDto, AppError> {
    let now = stopwatch::now_ms()?;
    let conn = state.lock_db()?;
    let t = db::rename_timer(&conn, id, name, now)?;
    Ok(t.to_dto())
}

/// Helper internal untuk mengubah durasi target timer dan mereset sisa waktu.
pub fn set_timer_duration_inner(
    state: &AppState,
    id: i64,
    duration_ms: i64,
) -> Result<TimerDto, AppError> {
    let now = stopwatch::now_ms()?;
    let conn = state.lock_db()?;
    let mut t = db::get_timer(&conn, id)?;
    timer::set_duration(&mut t, duration_ms, now)?;
    db::save_timer_state(&conn, &t)?;
    Ok(t.to_dto())
}

/// Helper internal untuk menjalankan (start/resume) timer secara idempotent.
pub fn start_timer_inner(state: &AppState, id: i64) -> Result<TimerDto, AppError> {
    let now = stopwatch::now_ms()?;
    let conn = state.lock_db()?;
    let mut t = db::get_timer(&conn, id)?;
    if t.started_at_ms.is_none() {
        timer::start(&mut t, now);
        db::save_timer_state(&conn, &t)?;
    }
    Ok(t.to_dto())
}

/// Helper internal untuk menjeda (pause) timer secara idempotent.
pub fn pause_timer_inner(state: &AppState, id: i64) -> Result<TimerDto, AppError> {
    let now = stopwatch::now_ms()?;
    let conn = state.lock_db()?;
    let mut t = db::get_timer(&conn, id)?;
    if t.started_at_ms.is_some() {
        timer::pause(&mut t, now);
        db::save_timer_state(&conn, &t)?;
    }
    Ok(t.to_dto())
}

/// Helper internal untuk menandai timer telah selesai (`remaining_ms = 0`).
pub fn complete_timer_inner(state: &AppState, id: i64) -> Result<TimerDto, AppError> {
    let now = stopwatch::now_ms()?;
    let conn = state.lock_db()?;
    let mut t = db::get_timer(&conn, id)?;
    timer::complete(&mut t, now);
    db::save_timer_state(&conn, &t)?;
    Ok(t.to_dto())
}

/// Helper internal untuk mereset timer kembali ke `duration_ms` (`Idle`).
pub fn reset_timer_inner(state: &AppState, id: i64) -> Result<TimerDto, AppError> {
    let now = stopwatch::now_ms()?;
    let conn = state.lock_db()?;
    let mut t = db::get_timer(&conn, id)?;
    timer::reset(&mut t, now);
    db::save_timer_state(&conn, &t)?;
    Ok(t.to_dto())
}

/// Helper internal untuk menghapus timer berdasarkan ID.
pub fn delete_timer_inner(state: &AppState, id: i64) -> Result<(), AppError> {
    let conn = state.lock_db()?;
    db::delete_timer(&conn, id)
}

#[tauri::command]
pub fn list_timers(state: State<'_, AppState>) -> Result<Vec<TimerDto>, String> {
    list_timers_inner(&state).map_err(Into::into)
}

#[tauri::command]
pub fn create_timer(
    state: State<'_, AppState>,
    name: Option<String>,
    duration_ms: Option<i64>,
) -> Result<TimerDto, String> {
    create_timer_inner(&state, name.as_deref(), duration_ms).map_err(Into::into)
}

#[tauri::command]
pub fn rename_timer(
    state: State<'_, AppState>,
    id: i64,
    name: String,
) -> Result<TimerDto, String> {
    rename_timer_inner(&state, id, &name).map_err(Into::into)
}

#[tauri::command]
pub fn set_timer_duration(
    state: State<'_, AppState>,
    id: i64,
    duration_ms: i64,
) -> Result<TimerDto, String> {
    set_timer_duration_inner(&state, id, duration_ms).map_err(Into::into)
}

#[tauri::command]
pub fn start_timer(state: State<'_, AppState>, id: i64) -> Result<TimerDto, String> {
    start_timer_inner(&state, id).map_err(Into::into)
}

#[tauri::command]
pub fn pause_timer(state: State<'_, AppState>, id: i64) -> Result<TimerDto, String> {
    pause_timer_inner(&state, id).map_err(Into::into)
}

#[tauri::command]
pub fn complete_timer(state: State<'_, AppState>, id: i64) -> Result<TimerDto, String> {
    complete_timer_inner(&state, id).map_err(Into::into)
}

#[tauri::command]
pub fn reset_timer(state: State<'_, AppState>, id: i64) -> Result<TimerDto, String> {
    reset_timer_inner(&state, id).map_err(Into::into)
}

#[tauri::command]
pub fn delete_timer(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    delete_timer_inner(&state, id).map_err(Into::into)
}

/// Helper internal untuk mengambil seluruh daftar alarm sebagai DTO.
pub fn list_alarms_inner(state: &AppState) -> Result<Vec<AlarmDto>, AppError> {
    let conn = state.lock_db()?;
    let items = db::list_alarms(&conn)?;
    Ok(items.iter().map(Alarm::to_dto).collect())
}

/// Helper internal untuk membuat alarm baru.
///
/// Jika jam dan menit tidak diberikan, alarm dijadwalkan pada jam berikutnya.
pub fn create_alarm_inner(
    state: &AppState,
    name: Option<&str>,
    hour: Option<i64>,
    minute: Option<i64>,
    repeat_days: Option<i64>,
) -> Result<AlarmDto, AppError> {
    let now = stopwatch::now_ms()?;
    let (valid_hour, valid_minute) = alarm::resolve_create_time(hour, minute, now)?;
    let valid_repeat = match repeat_days {
        Some(mask) => alarm::validate_repeat_days(mask)?,
        None => 0,
    };
    let conn = state.lock_db()?;
    let created = db::create_alarm(&conn, name, valid_hour, valid_minute, valid_repeat, now)?;
    Ok(created.to_dto())
}

/// Helper internal untuk memperbarui label, waktu, dan hari pengulangan alarm.
pub fn update_alarm_inner(
    state: &AppState,
    id: i64,
    name: &str,
    hour: i64,
    minute: i64,
    repeat_days: i64,
) -> Result<AlarmDto, AppError> {
    let valid_hour = alarm::validate_hour(hour)?;
    let valid_minute = alarm::validate_minute(minute)?;
    let valid_repeat = alarm::validate_repeat_days(repeat_days)?;
    let now = stopwatch::now_ms()?;
    let conn = state.lock_db()?;
    let updated = db::update_alarm(&conn, id, name, valid_hour, valid_minute, valid_repeat, now)?;
    Ok(updated.to_dto())
}

/// Helper internal untuk mengaktifkan atau menonaktifkan alarm.
pub fn set_alarm_enabled_inner(
    state: &AppState,
    id: i64,
    enabled: bool,
) -> Result<AlarmDto, AppError> {
    let now = stopwatch::now_ms()?;
    let conn = state.lock_db()?;
    let mut existing = db::get_alarm(&conn, id)?;
    alarm::set_enabled(&mut existing, enabled, now)?;
    db::save_alarm_state(&conn, &existing)?;
    Ok(existing.to_dto())
}

/// Helper internal untuk menandai alarm mulai berbunyi.
pub fn ring_alarm_inner(state: &AppState, id: i64) -> Result<AlarmDto, AppError> {
    let now = stopwatch::now_ms()?;
    let conn = state.lock_db()?;
    let mut existing = db::get_alarm(&conn, id)?;
    alarm::start_ringing(&mut existing, now)?;
    db::save_alarm_state(&conn, &existing)?;
    Ok(existing.to_dto())
}

/// Helper internal untuk menunda alarm yang sedang berbunyi.
pub fn snooze_alarm_inner(
    state: &AppState,
    id: i64,
    minutes: Option<i64>,
) -> Result<AlarmDto, AppError> {
    let now = stopwatch::now_ms()?;
    let snooze_minutes = minutes.unwrap_or(alarm::DEFAULT_SNOOZE_MINUTES);
    let conn = state.lock_db()?;
    let mut existing = db::get_alarm(&conn, id)?;
    alarm::snooze(&mut existing, snooze_minutes, now)?;
    db::save_alarm_state(&conn, &existing)?;
    Ok(existing.to_dto())
}

/// Helper internal untuk menutup alarm yang sedang berbunyi.
pub fn dismiss_alarm_inner(state: &AppState, id: i64) -> Result<AlarmDto, AppError> {
    let now = stopwatch::now_ms()?;
    let conn = state.lock_db()?;
    let mut existing = db::get_alarm(&conn, id)?;
    alarm::dismiss(&mut existing, now)?;
    db::save_alarm_state(&conn, &existing)?;
    Ok(existing.to_dto())
}

/// Helper internal untuk menghapus alarm berdasarkan ID.
pub fn delete_alarm_inner(state: &AppState, id: i64) -> Result<(), AppError> {
    let conn = state.lock_db()?;
    db::delete_alarm(&conn, id)
}

#[tauri::command]
pub fn list_alarms(state: State<'_, AppState>) -> Result<Vec<AlarmDto>, String> {
    list_alarms_inner(&state).map_err(Into::into)
}

#[tauri::command]
pub fn create_alarm(
    state: State<'_, AppState>,
    name: Option<String>,
    hour: Option<i64>,
    minute: Option<i64>,
    repeat_days: Option<i64>,
) -> Result<AlarmDto, String> {
    create_alarm_inner(&state, name.as_deref(), hour, minute, repeat_days).map_err(Into::into)
}

#[tauri::command]
pub fn update_alarm(
    state: State<'_, AppState>,
    id: i64,
    name: String,
    hour: i64,
    minute: i64,
    repeat_days: i64,
) -> Result<AlarmDto, String> {
    update_alarm_inner(&state, id, &name, hour, minute, repeat_days).map_err(Into::into)
}

#[tauri::command]
pub fn set_alarm_enabled(
    state: State<'_, AppState>,
    id: i64,
    enabled: bool,
) -> Result<AlarmDto, String> {
    set_alarm_enabled_inner(&state, id, enabled).map_err(Into::into)
}

#[tauri::command]
pub fn ring_alarm(state: State<'_, AppState>, id: i64) -> Result<AlarmDto, String> {
    ring_alarm_inner(&state, id).map_err(Into::into)
}

#[tauri::command]
pub fn snooze_alarm(
    state: State<'_, AppState>,
    id: i64,
    minutes: Option<i64>,
) -> Result<AlarmDto, String> {
    snooze_alarm_inner(&state, id, minutes).map_err(Into::into)
}

#[tauri::command]
pub fn dismiss_alarm(state: State<'_, AppState>, id: i64) -> Result<AlarmDto, String> {
    dismiss_alarm_inner(&state, id).map_err(Into::into)
}

#[tauri::command]
pub fn delete_alarm(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    delete_alarm_inner(&state, id).map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::StopwatchStatus;

    fn make_test_state() -> (tempfile::TempDir, AppState) {
        let dir = tempfile::tempdir().expect("create temp dir");
        let db_path = dir.path().join("test-commands.db");
        let conn = db::init_db(&db_path).expect("init_db should succeed");
        (dir, AppState::new(conn))
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

        // Jika sw1 sedang menjadi fokus, penghapusan sw1 wajib membersihkan focused_stopwatch_id
        {
            let mut focus = state.lock_focus().expect("lock focus");
            focus.focused_stopwatch_id = Some(sw1.id);
        }

        delete_stopwatch_inner(&state, sw1.id).expect("delete sw1");
        {
            let focus = state.lock_focus().expect("lock focus after delete");
            assert_eq!(focus.focused_stopwatch_id, None);
        }

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

    #[test]
    fn test_commands_alarm_crud_ring_snooze_and_validation() {
        let (_dir, state) = make_test_state();
        let before = stopwatch::now_ms().expect("current time");

        let created = create_alarm_inner(&state, None, None, None, None).expect("create alarm");
        assert_eq!(created.name, "Alarm 1");
        assert_eq!(created.position, 1);
        assert!(created.enabled);
        assert!(created.next_ring_at_ms.is_some_and(|next| next > before));

        assert_eq!(list_alarms_inner(&state).expect("list alarms").len(), 1);

        let updated = update_alarm_inner(&state, created.id, "  Pagi  ", 6, 15, 0b0001_1111)
            .expect("update alarm");
        assert_eq!(updated.name, "Pagi");
        assert_eq!(updated.hour, 6);
        assert_eq!(updated.minute, 15);
        assert_eq!(updated.repeat_days, 0b0001_1111);

        let disabled = set_alarm_enabled_inner(&state, created.id, false).expect("disable alarm");
        assert!(!disabled.enabled);
        assert_eq!(disabled.next_ring_at_ms, None);

        let enabled = set_alarm_enabled_inner(&state, created.id, true).expect("enable alarm");
        assert!(enabled.enabled);
        assert!(enabled.next_ring_at_ms.is_some());

        let snoozed = snooze_alarm_inner(&state, created.id, Some(10)).expect("snooze alarm");
        assert!(snoozed.enabled);
        assert!(snoozed.snoozed_until_ms.is_some());
        assert_eq!(snoozed.next_ring_at_ms, snoozed.snoozed_until_ms);

        assert!(snoozed.snoozed_until_ms.is_some());
        let ringing = ring_alarm_inner(&state, created.id).expect("ring alarm");
        assert!(ringing.enabled);
        assert_eq!(ringing.snoozed_until_ms, None);
        assert!(ringing.next_ring_at_ms.is_some());

        let dismissed = dismiss_alarm_inner(&state, created.id).expect("dismiss alarm");
        assert!(dismissed.enabled);
        assert_eq!(dismissed.snoozed_until_ms, None);

        let once = create_alarm_inner(&state, Some("Sekali"), Some(23), Some(59), Some(0))
            .expect("create one-shot alarm");
        let once_ringing = ring_alarm_inner(&state, once.id).expect("ring one-shot");
        assert!(!once_ringing.enabled);
        assert_eq!(once_ringing.next_ring_at_ms, None);

        assert!(matches!(
            create_alarm_inner(&state, None, Some(24), Some(0), None),
            Err(AppError::InvalidAlarmTime)
        ));
        assert!(matches!(
            create_alarm_inner(&state, None, Some(7), Some(0), Some(128)),
            Err(AppError::InvalidAlarmRepeatDays)
        ));
        assert!(matches!(
            update_alarm_inner(&state, created.id, "Pagi", 6, 60, 0),
            Err(AppError::InvalidAlarmTime)
        ));
        assert!(matches!(
            snooze_alarm_inner(&state, created.id, Some(0)),
            Err(AppError::InvalidSnoozeDuration)
        ));
        assert!(matches!(
            snooze_alarm_inner(&state, created.id, Some(61)),
            Err(AppError::InvalidSnoozeDuration)
        ));
        assert!(matches!(
            set_alarm_enabled_inner(&state, 9999, true),
            Err(AppError::AlarmNotFound(9999))
        ));

        delete_alarm_inner(&state, created.id).expect("delete alarm");
        assert!(matches!(
            delete_alarm_inner(&state, created.id),
            Err(AppError::AlarmNotFound(id)) if id == created.id
        ));
    }
}
