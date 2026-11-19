use std::path::Path;

use rusqlite::{Connection, ErrorCode, OptionalExtension, Row, params};

use crate::alarm;
use crate::error::AppError;
use crate::models::{Alarm, Stopwatch, Timer};
use crate::stopwatch;
use crate::timer;

/// Memetakan error saat inisialisasi database menjadi `AppError::DatabaseCorrupted`
/// apabila berkas SQLite rusak atau bukan berkas database yang valid.
fn map_db_init_error(db_path: &Path, err: impl Into<AppError>) -> AppError {
    let app_err = err.into();
    if let AppError::Database(ref db_err) = app_err
        && let rusqlite::Error::SqliteFailure(ref sqlite_err, _) = *db_err
        && (sqlite_err.code == ErrorCode::NotADatabase
            || sqlite_err.code == ErrorCode::DatabaseCorrupt)
    {
        return AppError::DatabaseCorrupted {
            path: db_path.display().to_string(),
            reason: db_err.to_string(),
        };
    }
    app_err
}

/// Mengatur PRAGMA koneksi SQLite agar aman dan mendukung konkurensi WAL.
fn configure_connection(conn: &Connection) -> Result<(), AppError> {
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA foreign_keys = ON;
         PRAGMA synchronous = NORMAL;
         PRAGMA busy_timeout = 5000;",
    )?;
    Ok(())
}

/// Menjalankan migrasi database berbasis `PRAGMA user_version`.
pub fn migrate(conn: &Connection) -> Result<(), AppError> {
    let version = get_user_version(conn)?;
    if version < 1 {
        conn.execute_batch(
            "BEGIN;
             CREATE TABLE IF NOT EXISTS stopwatches (
                 id             INTEGER PRIMARY KEY AUTOINCREMENT,
                 name           TEXT    NOT NULL,
                 accumulated_ms INTEGER NOT NULL DEFAULT 0,
                 started_at_ms  INTEGER,
                 heartbeat_ms   INTEGER,
                 position       INTEGER NOT NULL,
                 created_at_ms  INTEGER NOT NULL,
                 updated_at_ms  INTEGER NOT NULL
             );
             CREATE INDEX IF NOT EXISTS idx_stopwatches_position ON stopwatches(position ASC, id ASC);
             PRAGMA user_version = 1;
             COMMIT;",
        )?;
    }
    if version < 2 {
        conn.execute_batch(
            "BEGIN;
             CREATE TABLE IF NOT EXISTS timers (
                 id             INTEGER PRIMARY KEY AUTOINCREMENT,
                 name           TEXT    NOT NULL,
                 duration_ms    INTEGER NOT NULL,
                 remaining_ms   INTEGER NOT NULL,
                 started_at_ms  INTEGER,
                 heartbeat_ms   INTEGER,
                 position       INTEGER NOT NULL,
                 created_at_ms  INTEGER NOT NULL,
                 updated_at_ms  INTEGER NOT NULL
             );
             CREATE INDEX IF NOT EXISTS idx_timers_position ON timers(position ASC, id ASC);
             PRAGMA user_version = 2;
             COMMIT;",
        )?;
    }
    if version < 3 {
        conn.execute_batch(
            "BEGIN;
             CREATE TABLE IF NOT EXISTS alarms (
                 id                   INTEGER PRIMARY KEY AUTOINCREMENT,
                 name                 TEXT    NOT NULL,
                 hour                 INTEGER NOT NULL,
                 minute               INTEGER NOT NULL,
                 repeat_days          INTEGER NOT NULL DEFAULT 0,
                 enabled              INTEGER NOT NULL DEFAULT 1,
                 snoozed_until_ms     INTEGER,
                 next_ring_at_ms      INTEGER,
                 last_triggered_at_ms INTEGER,
                 position             INTEGER NOT NULL,
                 created_at_ms        INTEGER NOT NULL,
                 updated_at_ms        INTEGER NOT NULL
             );
             CREATE INDEX IF NOT EXISTS idx_alarms_position ON alarms(position ASC, id ASC);
             PRAGMA user_version = 3;
             COMMIT;",
        )?;
    }
    Ok(())
}

/// Membuka atau membuat database SQLite pada `db_path`, mengatur PRAGMA, dan menjalankan migrasi.
pub fn init_db(db_path: &Path) -> Result<Connection, AppError> {
    if let Some(parent) = db_path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)?;
    }
    let conn = Connection::open(db_path).map_err(|err| map_db_init_error(db_path, err))?;
    configure_connection(&conn).map_err(|err| map_db_init_error(db_path, err))?;
    migrate(&conn).map_err(|err| map_db_init_error(db_path, err))?;
    Ok(conn)
}

/// Membaca nilai `PRAGMA user_version` saat ini.
pub fn get_user_version(conn: &Connection) -> Result<i32, AppError> {
    let version: i32 = conn.query_row("PRAGMA user_version", params![], |row| row.get(0))?;
    Ok(version)
}

fn map_stopwatch_row(row: &Row<'_>) -> Result<Stopwatch, rusqlite::Error> {
    Ok(Stopwatch {
        id: row.get(0)?,
        name: row.get(1)?,
        accumulated_ms: row.get(2)?,
        started_at_ms: row.get(3)?,
        heartbeat_ms: row.get(4)?,
        position: row.get(5)?,
        created_at_ms: row.get(6)?,
        updated_at_ms: row.get(7)?,
    })
}

/// Mengambil seluruh daftar stopwatch diurutkan berdasarkan `position ASC, id ASC`.
pub fn list_stopwatches(conn: &Connection) -> Result<Vec<Stopwatch>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, accumulated_ms, started_at_ms, heartbeat_ms, position, created_at_ms, updated_at_ms
         FROM stopwatches
         ORDER BY position ASC, id ASC",
    )?;
    let rows = stmt.query_map(params![], map_stopwatch_row)?;
    let mut items = Vec::new();
    for row in rows {
        items.push(row?);
    }
    Ok(items)
}

/// Mengambil satu stopwatch berdasarkan `id`.
pub fn get_stopwatch(conn: &Connection, id: i64) -> Result<Stopwatch, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, accumulated_ms, started_at_ms, heartbeat_ms, position, created_at_ms, updated_at_ms
         FROM stopwatches
         WHERE id = ?1",
    )?;
    let sw = stmt
        .query_row(params![id], map_stopwatch_row)
        .optional()?
        .ok_or(AppError::NotFound(id))?;
    Ok(sw)
}

/// Menghitung angka `N` berikutnya untuk penamaan default `"Stopwatch N"` dan `position`.
pub fn next_default_number(conn: &Connection) -> Result<i64, AppError> {
    let next_pos: i64 = conn.query_row(
        "SELECT COALESCE(MAX(position), 0) + 1 FROM stopwatches",
        params![],
        |row| row.get(0),
    )?;
    Ok(next_pos.max(1))
}

/// Membuat stopwatch baru di database.
pub fn create_stopwatch(
    conn: &Connection,
    name: Option<&str>,
    now_ms: i64,
) -> Result<Stopwatch, AppError> {
    let position = next_default_number(conn)?;
    let resolved_name = stopwatch::resolve_create_name(name, position)?;

    conn.execute(
        "INSERT INTO stopwatches (name, accumulated_ms, started_at_ms, heartbeat_ms, position, created_at_ms, updated_at_ms)
         VALUES (?1, 0, NULL, NULL, ?2, ?3, ?3)",
        params![resolved_name, position, now_ms],
    )?;

    let id = conn.last_insert_rowid();
    Ok(Stopwatch {
        id,
        name: resolved_name,
        accumulated_ms: 0,
        started_at_ms: None,
        heartbeat_ms: None,
        position,
        created_at_ms: now_ms,
        updated_at_ms: now_ms,
    })
}

/// Mengubah nama stopwatch yang sudah ada.
pub fn rename_stopwatch(
    conn: &Connection,
    id: i64,
    new_name: &str,
    now_ms: i64,
) -> Result<Stopwatch, AppError> {
    let validated = stopwatch::validate_name(new_name)?;
    let affected = conn.execute(
        "UPDATE stopwatches SET name = ?1, updated_at_ms = ?2 WHERE id = ?3",
        params![validated, now_ms, id],
    )?;
    if affected == 0 {
        return Err(AppError::NotFound(id));
    }
    get_stopwatch(conn, id)
}

/// Menyimpan perubahan state waktu (`accumulated_ms`, `started_at_ms`, `heartbeat_ms`, `updated_at_ms`) ke database.
pub fn save_stopwatch_state(conn: &Connection, sw: &Stopwatch) -> Result<(), AppError> {
    let affected = conn.execute(
        "UPDATE stopwatches
         SET accumulated_ms = ?1, started_at_ms = ?2, heartbeat_ms = ?3, updated_at_ms = ?4
         WHERE id = ?5",
        params![
            sw.accumulated_ms,
            sw.started_at_ms,
            sw.heartbeat_ms,
            sw.updated_at_ms,
            sw.id
        ],
    )?;
    if affected == 0 {
        return Err(AppError::NotFound(sw.id));
    }
    Ok(())
}

/// Menghapus stopwatch berdasarkan `id`.
pub fn delete_stopwatch(conn: &Connection, id: i64) -> Result<(), AppError> {
    let affected = conn.execute("DELETE FROM stopwatches WHERE id = ?1", params![id])?;
    if affected == 0 {
        return Err(AppError::NotFound(id));
    }
    Ok(())
}

/// Menjeda seluruh stopwatch dan timer yang sedang berjalan dalam satu transaksi SQLite.
pub fn pause_all_running(conn: &mut Connection, now_ms: i64) -> Result<usize, AppError> {
    let tx = conn.transaction()?;
    let running_items = {
        let mut stmt = tx.prepare(
            "SELECT id, name, accumulated_ms, started_at_ms, heartbeat_ms, position, created_at_ms, updated_at_ms
             FROM stopwatches
             WHERE started_at_ms IS NOT NULL",
        )?;
        let rows = stmt.query_map(params![], map_stopwatch_row)?;
        let mut list = Vec::new();
        for row in rows {
            list.push(row?);
        }
        list
    };

    let running_timers = {
        let mut stmt = tx.prepare(
            "SELECT id, name, duration_ms, remaining_ms, started_at_ms, heartbeat_ms, position, created_at_ms, updated_at_ms
             FROM timers
             WHERE started_at_ms IS NOT NULL",
        )?;
        let rows = stmt.query_map(params![], map_timer_row)?;
        let mut list = Vec::new();
        for row in rows {
            list.push(row?);
        }
        list
    };

    let count = running_items.len() + running_timers.len();
    for mut sw in running_items {
        stopwatch::pause(&mut sw, now_ms);
        tx.execute(
            "UPDATE stopwatches
             SET accumulated_ms = ?1, started_at_ms = ?2, heartbeat_ms = ?3, updated_at_ms = ?4
             WHERE id = ?5",
            params![
                sw.accumulated_ms,
                sw.started_at_ms,
                sw.heartbeat_ms,
                sw.updated_at_ms,
                sw.id
            ],
        )?;
    }

    for mut t in running_timers {
        timer::pause(&mut t, now_ms);
        tx.execute(
            "UPDATE timers
             SET remaining_ms = ?1, started_at_ms = ?2, heartbeat_ms = ?3, updated_at_ms = ?4
             WHERE id = ?5",
            params![
                t.remaining_ms,
                t.started_at_ms,
                t.heartbeat_ms,
                t.updated_at_ms,
                t.id
            ],
        )?;
    }

    tx.commit()?;
    Ok(count)
}

/// Memulihkan seluruh stopwatch dan timer yang masih berstatus running ketika aplikasi dibuka kembali setelah crash.
pub fn recover_crashed_running(conn: &mut Connection, now_ms: i64) -> Result<usize, AppError> {
    let tx = conn.transaction()?;
    let crashed_items = {
        let mut stmt = tx.prepare(
            "SELECT id, name, accumulated_ms, started_at_ms, heartbeat_ms, position, created_at_ms, updated_at_ms
             FROM stopwatches
             WHERE started_at_ms IS NOT NULL",
        )?;
        let rows = stmt.query_map(params![], map_stopwatch_row)?;
        let mut list = Vec::new();
        for row in rows {
            list.push(row?);
        }
        list
    };

    let crashed_timers = {
        let mut stmt = tx.prepare(
            "SELECT id, name, duration_ms, remaining_ms, started_at_ms, heartbeat_ms, position, created_at_ms, updated_at_ms
             FROM timers
             WHERE started_at_ms IS NOT NULL",
        )?;
        let rows = stmt.query_map(params![], map_timer_row)?;
        let mut list = Vec::new();
        for row in rows {
            list.push(row?);
        }
        list
    };

    let count = crashed_items.len() + crashed_timers.len();
    for mut sw in crashed_items {
        stopwatch::recover_from_crash(&mut sw, now_ms);
        tx.execute(
            "UPDATE stopwatches
             SET accumulated_ms = ?1, started_at_ms = ?2, heartbeat_ms = ?3, updated_at_ms = ?4
             WHERE id = ?5",
            params![
                sw.accumulated_ms,
                sw.started_at_ms,
                sw.heartbeat_ms,
                sw.updated_at_ms,
                sw.id
            ],
        )?;
    }

    for mut t in crashed_timers {
        timer::recover_from_crash(&mut t, now_ms);
        tx.execute(
            "UPDATE timers
             SET remaining_ms = ?1, started_at_ms = ?2, heartbeat_ms = ?3, updated_at_ms = ?4
             WHERE id = ?5",
            params![
                t.remaining_ms,
                t.started_at_ms,
                t.heartbeat_ms,
                t.updated_at_ms,
                t.id
            ],
        )?;
    }

    tx.commit()?;
    Ok(count)
}

/// Memperbarui nilai `heartbeat_ms` untuk semua stopwatch dan timer yang sedang berjalan.
pub fn update_running_heartbeats(conn: &Connection, now_ms: i64) -> Result<usize, AppError> {
    let sw_affected = conn.execute(
        "UPDATE stopwatches
         SET heartbeat_ms = ?1, updated_at_ms = ?1
         WHERE started_at_ms IS NOT NULL",
        params![now_ms],
    )?;
    let timer_affected = conn.execute(
        "UPDATE timers
         SET heartbeat_ms = ?1, updated_at_ms = ?1
         WHERE started_at_ms IS NOT NULL",
        params![now_ms],
    )?;
    Ok(sw_affected + timer_affected)
}

fn map_timer_row(row: &Row<'_>) -> Result<Timer, rusqlite::Error> {
    Ok(Timer {
        id: row.get(0)?,
        name: row.get(1)?,
        duration_ms: row.get(2)?,
        remaining_ms: row.get(3)?,
        started_at_ms: row.get(4)?,
        heartbeat_ms: row.get(5)?,
        position: row.get(6)?,
        created_at_ms: row.get(7)?,
        updated_at_ms: row.get(8)?,
    })
}

/// Mengambil seluruh daftar timer diurutkan berdasarkan `position ASC, id ASC`.
pub fn list_timers(conn: &Connection) -> Result<Vec<Timer>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, duration_ms, remaining_ms, started_at_ms, heartbeat_ms, position, created_at_ms, updated_at_ms
         FROM timers
         ORDER BY position ASC, id ASC",
    )?;
    let rows = stmt.query_map(params![], map_timer_row)?;
    let mut items = Vec::new();
    for row in rows {
        items.push(row?);
    }
    Ok(items)
}

/// Mengambil satu timer berdasarkan `id`.
pub fn get_timer(conn: &Connection, id: i64) -> Result<Timer, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, duration_ms, remaining_ms, started_at_ms, heartbeat_ms, position, created_at_ms, updated_at_ms
         FROM timers
         WHERE id = ?1",
    )?;
    let t = stmt
        .query_row(params![id], map_timer_row)
        .optional()?
        .ok_or(AppError::TimerNotFound(id))?;
    Ok(t)
}

/// Menghitung angka `N` berikutnya untuk penamaan default `"Timer N"` dan `position`.
pub fn next_timer_default_number(conn: &Connection) -> Result<i64, AppError> {
    let next_pos: i64 = conn.query_row(
        "SELECT COALESCE(MAX(position), 0) + 1 FROM timers",
        params![],
        |row| row.get(0),
    )?;
    Ok(next_pos.max(1))
}

/// Membuat timer baru di database.
pub fn create_timer(
    conn: &Connection,
    name: Option<&str>,
    duration_ms: Option<i64>,
    now_ms: i64,
) -> Result<Timer, AppError> {
    let position = next_timer_default_number(conn)?;
    let resolved_name = timer::resolve_create_name(name, position)?;
    let valid_duration = timer::validate_duration(duration_ms.unwrap_or(timer::DEFAULT_DURATION_MS))?;

    conn.execute(
        "INSERT INTO timers (name, duration_ms, remaining_ms, started_at_ms, heartbeat_ms, position, created_at_ms, updated_at_ms)
         VALUES (?1, ?2, ?2, NULL, NULL, ?3, ?4, ?4)",
        params![resolved_name, valid_duration, position, now_ms],
    )?;

    let id = conn.last_insert_rowid();
    Ok(Timer {
        id,
        name: resolved_name,
        duration_ms: valid_duration,
        remaining_ms: valid_duration,
        started_at_ms: None,
        heartbeat_ms: None,
        position,
        created_at_ms: now_ms,
        updated_at_ms: now_ms,
    })
}

/// Mengubah nama timer yang sudah ada.
pub fn rename_timer(
    conn: &Connection,
    id: i64,
    new_name: &str,
    now_ms: i64,
) -> Result<Timer, AppError> {
    let validated = stopwatch::validate_name(new_name)?;
    let affected = conn.execute(
        "UPDATE timers SET name = ?1, updated_at_ms = ?2 WHERE id = ?3",
        params![validated, now_ms, id],
    )?;
    if affected == 0 {
        return Err(AppError::TimerNotFound(id));
    }
    get_timer(conn, id)
}

/// Menyimpan perubahan state waktu timer (`duration_ms`, `remaining_ms`, `started_at_ms`, `heartbeat_ms`, `updated_at_ms`) ke database.
pub fn save_timer_state(conn: &Connection, t: &Timer) -> Result<(), AppError> {
    let affected = conn.execute(
        "UPDATE timers
         SET duration_ms = ?1, remaining_ms = ?2, started_at_ms = ?3, heartbeat_ms = ?4, updated_at_ms = ?5
         WHERE id = ?6",
        params![
            t.duration_ms,
            t.remaining_ms,
            t.started_at_ms,
            t.heartbeat_ms,
            t.updated_at_ms,
            t.id
        ],
    )?;
    if affected == 0 {
        return Err(AppError::TimerNotFound(t.id));
    }
    Ok(())
}

/// Menghapus timer berdasarkan `id`.
pub fn delete_timer(conn: &Connection, id: i64) -> Result<(), AppError> {
    let affected = conn.execute("DELETE FROM timers WHERE id = ?1", params![id])?;
    if affected == 0 {
        return Err(AppError::TimerNotFound(id));
    }
    Ok(())
}

fn map_alarm_row(row: &Row<'_>) -> Result<Alarm, rusqlite::Error> {
    Ok(Alarm {
        id: row.get(0)?,
        name: row.get(1)?,
        hour: row.get(2)?,
        minute: row.get(3)?,
        repeat_days: row.get(4)?,
        enabled: row.get(5)?,
        snoozed_until_ms: row.get(6)?,
        next_ring_at_ms: row.get(7)?,
        last_triggered_at_ms: row.get(8)?,
        position: row.get(9)?,
        created_at_ms: row.get(10)?,
        updated_at_ms: row.get(11)?,
    })
}

/// Mengambil seluruh daftar alarm diurutkan berdasarkan `position ASC, id ASC`.
pub fn list_alarms(conn: &Connection) -> Result<Vec<Alarm>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, hour, minute, repeat_days, enabled, snoozed_until_ms, next_ring_at_ms,
                last_triggered_at_ms, position, created_at_ms, updated_at_ms
         FROM alarms
         ORDER BY position ASC, id ASC",
    )?;
    let rows = stmt.query_map(params![], map_alarm_row)?;
    let mut items = Vec::new();
    for row in rows {
        items.push(row?);
    }
    Ok(items)
}

/// Mengambil satu alarm berdasarkan `id`.
pub fn get_alarm(conn: &Connection, id: i64) -> Result<Alarm, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, hour, minute, repeat_days, enabled, snoozed_until_ms, next_ring_at_ms,
                last_triggered_at_ms, position, created_at_ms, updated_at_ms
         FROM alarms
         WHERE id = ?1",
    )?;
    let alarm = stmt
        .query_row(params![id], map_alarm_row)
        .optional()?
        .ok_or(AppError::AlarmNotFound(id))?;
    Ok(alarm)
}

/// Menghitung angka `N` berikutnya untuk penamaan default `"Alarm N"` dan `position`.
pub fn next_alarm_default_number(conn: &Connection) -> Result<i64, AppError> {
    let next_pos: i64 = conn.query_row(
        "SELECT COALESCE(MAX(position), 0) + 1 FROM alarms",
        params![],
        |row| row.get(0),
    )?;
    Ok(next_pos.max(1))
}

/// Membuat alarm baru beserta jadwal berbunyi pertamanya.
pub fn create_alarm(
    conn: &Connection,
    name: Option<&str>,
    hour: u8,
    minute: u8,
    repeat_days: u8,
    now_ms: i64,
) -> Result<Alarm, AppError> {
    let valid_hour = alarm::validate_hour(i64::from(hour))?;
    let valid_minute = alarm::validate_minute(i64::from(minute))?;
    let valid_repeat = alarm::validate_repeat_days(i64::from(repeat_days))?;
    let position = next_alarm_default_number(conn)?;
    let resolved_name = alarm::resolve_create_name(name, position)?;

    let mut created = Alarm {
        id: 0,
        name: resolved_name,
        hour: valid_hour,
        minute: valid_minute,
        repeat_days: valid_repeat,
        enabled: true,
        snoozed_until_ms: None,
        next_ring_at_ms: None,
        last_triggered_at_ms: None,
        position,
        created_at_ms: now_ms,
        updated_at_ms: now_ms,
    };
    alarm::schedule(&mut created, now_ms)?;

    conn.execute(
        "INSERT INTO alarms (name, hour, minute, repeat_days, enabled, snoozed_until_ms,
                             next_ring_at_ms, last_triggered_at_ms, position, created_at_ms, updated_at_ms)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            created.name,
            created.hour,
            created.minute,
            created.repeat_days,
            created.enabled,
            created.snoozed_until_ms,
            created.next_ring_at_ms,
            created.last_triggered_at_ms,
            created.position,
            created.created_at_ms,
            created.updated_at_ms
        ],
    )?;

    created.id = conn.last_insert_rowid();
    Ok(created)
}

/// Menyimpan seluruh perubahan state alarm ke database.
pub fn save_alarm_state(conn: &Connection, alarm: &Alarm) -> Result<(), AppError> {
    let affected = conn.execute(
        "UPDATE alarms
         SET name = ?1, hour = ?2, minute = ?3, repeat_days = ?4, enabled = ?5,
             snoozed_until_ms = ?6, next_ring_at_ms = ?7, last_triggered_at_ms = ?8,
             updated_at_ms = ?9
         WHERE id = ?10",
        params![
            alarm.name,
            alarm.hour,
            alarm.minute,
            alarm.repeat_days,
            alarm.enabled,
            alarm.snoozed_until_ms,
            alarm.next_ring_at_ms,
            alarm.last_triggered_at_ms,
            alarm.updated_at_ms,
            alarm.id
        ],
    )?;
    if affected == 0 {
        return Err(AppError::AlarmNotFound(alarm.id));
    }
    Ok(())
}

/// Memperbarui label, waktu, dan hari pengulangan alarm.
///
/// Perubahan jadwal membatalkan snooze yang sedang tertunda agar alarm tidak
/// berbunyi pada waktu lama yang sudah tidak relevan.
pub fn update_alarm(
    conn: &Connection,
    id: i64,
    name: &str,
    hour: u8,
    minute: u8,
    repeat_days: u8,
    now_ms: i64,
) -> Result<Alarm, AppError> {
    let valid_name = stopwatch::validate_name(name)?;
    let valid_hour = alarm::validate_hour(i64::from(hour))?;
    let valid_minute = alarm::validate_minute(i64::from(minute))?;
    let valid_repeat = alarm::validate_repeat_days(i64::from(repeat_days))?;

    let mut existing = get_alarm(conn, id)?;
    existing.name = valid_name;
    existing.hour = valid_hour;
    existing.minute = valid_minute;
    existing.repeat_days = valid_repeat;
    existing.snoozed_until_ms = None;
    existing.updated_at_ms = now_ms;
    if existing.enabled {
        alarm::schedule(&mut existing, now_ms)?;
    } else {
        existing.next_ring_at_ms = None;
    }
    save_alarm_state(conn, &existing)?;
    Ok(existing)
}

/// Menghapus alarm berdasarkan `id`.
pub fn delete_alarm(conn: &Connection, id: i64) -> Result<(), AppError> {
    let affected = conn.execute("DELETE FROM alarms WHERE id = ?1", params![id])?;
    if affected == 0 {
        return Err(AppError::AlarmNotFound(id));
    }
    Ok(())
}

/// Menyegarkan jadwal seluruh alarm saat aplikasi dibuka.
///
/// - Snooze yang tertunda dipertahankan, termasuk yang sudah lewat agar langsung berbunyi.
/// - Alarm sekali pakai yang terlewat saat aplikasi tertutup dinonaktifkan.
/// - Alarm berulang dijadwalkan ulang dari waktu sekarang.
pub fn rearm_alarms(conn: &Connection, now_ms: i64) -> Result<usize, AppError> {
    let mut changed = 0;

    for mut scheduled in list_alarms(conn)? {
        if !scheduled.enabled {
            if scheduled.next_ring_at_ms.is_some() || scheduled.snoozed_until_ms.is_some() {
                scheduled.next_ring_at_ms = None;
                scheduled.snoozed_until_ms = None;
                save_alarm_state(conn, &scheduled)?;
                changed += 1;
            }
            continue;
        }

        if let Some(snoozed_until) = scheduled.snoozed_until_ms {
            if scheduled.next_ring_at_ms != Some(snoozed_until) {
                scheduled.next_ring_at_ms = Some(snoozed_until);
                save_alarm_state(conn, &scheduled)?;
                changed += 1;
            }
            continue;
        }

        let missed_once =
            scheduled.repeat_days == 0 && scheduled.next_ring_at_ms.is_some_and(|t| t <= now_ms);
        if missed_once {
            scheduled.enabled = false;
            scheduled.next_ring_at_ms = None;
            scheduled.updated_at_ms = now_ms;
            save_alarm_state(conn, &scheduled)?;
            changed += 1;
            continue;
        }

        let next_ring = alarm::compute_regular_next_ring_at_ms(
            scheduled.hour,
            scheduled.minute,
            scheduled.repeat_days,
            now_ms,
        )?;
        if scheduled.next_ring_at_ms != next_ring {
            scheduled.next_ring_at_ms = next_ring;
            save_alarm_state(conn, &scheduled)?;
            changed += 1;
        }
    }

    Ok(changed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::StopwatchStatus;

    fn open_test_memory_db() -> Connection {
        let conn = Connection::open_in_memory().expect("in-memory sqlite should open");
        configure_connection(&conn).expect("pragma configuration should succeed");
        migrate(&conn).expect("migration should succeed");
        conn
    }

    #[test]
    fn test_migration_sets_user_version_3() {
        let conn = open_test_memory_db();
        assert_eq!(get_user_version(&conn).ok(), Some(3));

        // Memastikan idempotent jika migrate dipanggil lagi
        assert!(migrate(&conn).is_ok());
        assert_eq!(get_user_version(&conn).ok(), Some(3));
    }

    #[test]
    fn test_crud_lifecycle_create_list_rename_delete() {
        let conn = open_test_memory_db();

        let sw1 = create_stopwatch(&conn, None, 1_000).expect("create default stopwatch");
        assert_eq!(sw1.name, "Stopwatch 1");
        assert_eq!(sw1.position, 1);

        let sw2 = create_stopwatch(&conn, Some("  Belajar Rust  "), 1_500)
            .expect("create named stopwatch");
        assert_eq!(sw2.name, "Belajar Rust");
        assert_eq!(sw2.position, 2);

        let all = list_stopwatches(&conn).expect("list stopwatches");
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].id, sw1.id);
        assert_eq!(all[1].id, sw2.id);

        let renamed =
            rename_stopwatch(&conn, sw2.id, "Coding Tauri", 2_000).expect("rename stopwatch");
        assert_eq!(renamed.name, "Coding Tauri");
        assert_eq!(renamed.updated_at_ms, 2_000);

        delete_stopwatch(&conn, sw1.id).expect("delete first stopwatch");
        let remaining = list_stopwatches(&conn).expect("list after delete");
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].id, sw2.id);

        let second_delete = delete_stopwatch(&conn, sw1.id);
        assert!(matches!(second_delete, Err(AppError::NotFound(id)) if id == sw1.id));
    }

    #[test]
    fn test_start_pause_reset_persistence() {
        let conn = open_test_memory_db();
        let mut sw = create_stopwatch(&conn, Some("Olahraga"), 1_000).expect("create");

        stopwatch::start(&mut sw, 2_000);
        save_stopwatch_state(&conn, &sw).expect("save start");

        let loaded_running = get_stopwatch(&conn, sw.id).expect("get running");
        assert_eq!(loaded_running.status(), StopwatchStatus::Running);
        assert_eq!(loaded_running.started_at_ms, Some(2_000));
        assert_eq!(loaded_running.heartbeat_ms, Some(2_000));

        let updated_hb = update_running_heartbeats(&conn, 2_500).expect("update heartbeats");
        assert_eq!(updated_hb, 1);

        let mut sw_after_hb = get_stopwatch(&conn, sw.id).expect("get after hb");
        assert_eq!(sw_after_hb.heartbeat_ms, Some(2_500));

        stopwatch::pause(&mut sw_after_hb, 5_000);
        save_stopwatch_state(&conn, &sw_after_hb).expect("save pause");

        let loaded_paused = get_stopwatch(&conn, sw.id).expect("get paused");
        assert_eq!(loaded_paused.status(), StopwatchStatus::Paused);
        assert_eq!(loaded_paused.accumulated_ms, 3_000);
        assert_eq!(loaded_paused.started_at_ms, None);
        assert_eq!(loaded_paused.heartbeat_ms, None);

        let mut sw_to_reset = loaded_paused;
        stopwatch::reset(&mut sw_to_reset, 6_000);
        save_stopwatch_state(&conn, &sw_to_reset).expect("save reset");

        let loaded_idle = get_stopwatch(&conn, sw.id).expect("get idle");
        assert_eq!(loaded_idle.status(), StopwatchStatus::Idle);
        assert_eq!(loaded_idle.accumulated_ms, 0);
    }

    #[test]
    fn test_pause_all_running_in_single_transaction() {
        let mut conn = open_test_memory_db();

        let mut sw1 = create_stopwatch(&conn, Some("Satu"), 1_000).expect("create 1");
        let mut sw2 = create_stopwatch(&conn, Some("Dua"), 1_000).expect("create 2");
        let mut sw3 = create_stopwatch(&conn, Some("Tiga"), 1_000).expect("create 3");

        stopwatch::start(&mut sw1, 2_000);
        save_stopwatch_state(&conn, &sw1).expect("save sw1");

        stopwatch::start(&mut sw2, 3_000);
        save_stopwatch_state(&conn, &sw2).expect("save sw2");

        stopwatch::start(&mut sw3, 1_500);
        stopwatch::pause(&mut sw3, 2_500);
        save_stopwatch_state(&conn, &sw3).expect("save sw3");

        let paused_count = pause_all_running(&mut conn, 5_000).expect("pause all running");
        assert_eq!(paused_count, 2);

        let after1 = get_stopwatch(&conn, sw1.id).expect("get 1");
        let after2 = get_stopwatch(&conn, sw2.id).expect("get 2");
        let after3 = get_stopwatch(&conn, sw3.id).expect("get 3");

        assert_eq!(after1.status(), StopwatchStatus::Paused);
        assert_eq!(after1.accumulated_ms, 3_000);
        assert_eq!(after1.started_at_ms, None);

        assert_eq!(after2.status(), StopwatchStatus::Paused);
        assert_eq!(after2.accumulated_ms, 2_000);
        assert_eq!(after2.started_at_ms, None);

        assert_eq!(after3.status(), StopwatchStatus::Paused);
        assert_eq!(after3.accumulated_ms, 1_000);
    }

    #[test]
    fn test_persistence_across_connection_reopen() {
        let temp_dir = tempfile::tempdir().expect("create tempdir");
        let db_path = temp_dir.path().join("multi-stopwatch.db");

        let sw_id = {
            let conn = init_db(&db_path).expect("init_db first open");
            let mut sw = create_stopwatch(&conn, Some("Riset SQLite"), 10_000).expect("create");
            stopwatch::start(&mut sw, 11_000);
            stopwatch::pause(&mut sw, 15_500);
            save_stopwatch_state(&conn, &sw).expect("save state");
            sw.id
        };

        let mut conn2 = init_db(&db_path).expect("init_db reopen");
        assert_eq!(get_user_version(&conn2).ok(), Some(3));

        let loaded = get_stopwatch(&conn2, sw_id).expect("load persisted stopwatch");
        assert_eq!(loaded.name, "Riset SQLite");
        assert_eq!(loaded.accumulated_ms, 4_500);
        assert_eq!(loaded.started_at_ms, None);
        assert_eq!(loaded.status(), StopwatchStatus::Paused);

        // Menguji recover_crashed_running pada koneksi nyata
        let mut running_sw = loaded;
        stopwatch::start(&mut running_sw, 20_000);
        save_stopwatch_state(&conn2, &running_sw).expect("save running");
        update_running_heartbeats(&conn2, 35_000).expect("update hb");

        let recovered = recover_crashed_running(&mut conn2, 50_000).expect("recover crashed");
        assert_eq!(recovered, 1);

        let after_recover = get_stopwatch(&conn2, sw_id).expect("get after recover");
        assert_eq!(after_recover.accumulated_ms, 19_500);
        assert_eq!(after_recover.started_at_ms, None);
        assert_eq!(after_recover.status(), StopwatchStatus::Paused);
    }

    #[test]
    fn test_default_numbering_after_middle_deletion_and_duplicate_names() {
        let conn = open_test_memory_db();

        let sw1 = create_stopwatch(&conn, None, 1_000).expect("create 1");
        let sw2 = create_stopwatch(&conn, None, 1_100).expect("create 2");
        assert_eq!(sw1.name, "Stopwatch 1");
        assert_eq!(sw2.name, "Stopwatch 2");

        // Menghapus stopwatch pertama di tengah/awal tidak boleh membuat stopwatch berikutnya bentrok jadi "Stopwatch 2"
        delete_stopwatch(&conn, sw1.id).expect("delete sw1");

        let sw3 = create_stopwatch(&conn, None, 1_200).expect("create 3");
        assert_eq!(sw3.name, "Stopwatch 3");
        assert_eq!(sw3.position, 3);

        // Nama duplikat diizinkan sesuai spesifikasi F-02
        let dup =
            create_stopwatch(&conn, Some("Stopwatch 3"), 1_300).expect("create duplicate name");
        assert_eq!(dup.name, "Stopwatch 3");
        assert_eq!(dup.position, 4);
    }

    #[test]
    fn test_corrupted_database_file_returns_clear_error_and_preserves_file() {
        let temp_dir = tempfile::tempdir().expect("create tempdir");
        let db_path = temp_dir.path().join("multi-stopwatch.db");
        let corrupted_bytes = b"NOT_A_VALID_SQLITE_HEADER_DATA_1234567890";

        std::fs::write(&db_path, corrupted_bytes).expect("write corrupted bytes");

        let result = init_db(&db_path);
        match result {
            Err(AppError::DatabaseCorrupted { ref path, ref reason }) => {
                assert_eq!(path, &db_path.display().to_string());
                assert!(!reason.is_empty());
            }
            other => panic!("expected Err(AppError::DatabaseCorrupted), got: {other:?}"),
        }

        // Pastikan file asli di disk tidak dihapus dan isinya sama persis (tidak ditimpa)
        assert!(db_path.exists());
        let after_bytes = std::fs::read(&db_path).expect("read file after init_db failure");
        assert_eq!(after_bytes.as_slice(), corrupted_bytes);
    }

    #[test]
    fn test_timer_crud_and_persistence() {
        let mut conn = open_test_memory_db();

        let t1 = create_timer(&conn, None, None, 1_000).expect("create default timer");
        assert_eq!(t1.name, "Timer 1");
        assert_eq!(t1.duration_ms, timer::DEFAULT_DURATION_MS);
        assert_eq!(t1.remaining_ms, timer::DEFAULT_DURATION_MS);
        assert_eq!(t1.position, 1);

        let mut t2 = create_timer(&conn, Some("  Pomodoro  "), Some(150_000), 1_500)
            .expect("create named timer");
        assert_eq!(t2.name, "Pomodoro");
        assert_eq!(t2.duration_ms, 150_000);
        assert_eq!(t2.position, 2);

        let all = list_timers(&conn).expect("list timers");
        assert_eq!(all.len(), 2);

        let renamed = rename_timer(&conn, t2.id, "Fokus 25m", 2_000).expect("rename timer");
        assert_eq!(renamed.name, "Fokus 25m");

        timer::start(&mut t2, 3_000);
        save_timer_state(&conn, &t2).expect("save running timer");

        let paused_count = pause_all_running(&mut conn, 8_000).expect("pause all running");
        assert_eq!(paused_count, 1);

        let after_pause = get_timer(&conn, t2.id).expect("get paused timer");
        assert_eq!(after_pause.remaining_ms, 145_000);
        assert_eq!(after_pause.started_at_ms, None);

        delete_timer(&conn, t1.id).expect("delete t1");
        assert!(matches!(
            delete_timer(&conn, t1.id),
            Err(AppError::TimerNotFound(id)) if id == t1.id
        ));
    }

    #[test]
    fn test_alarm_crud_and_state_persistence() {
        let conn = open_test_memory_db();
        let now = stopwatch::now_ms().expect("current time");

        let created = create_alarm(&conn, None, 7, 30, 0, now).expect("create default alarm");
        assert_eq!(created.name, "Alarm 1");
        assert_eq!(created.position, 1);
        assert!(created.enabled);
        assert!(created.next_ring_at_ms.is_some_and(|next| next > now));

        let updated = update_alarm(
            &conn,
            created.id,
            "  Bangun pagi  ",
            6,
            45,
            0b0001_1111,
            now,
        )
        .expect("update alarm");
        assert_eq!(updated.name, "Bangun pagi");
        assert_eq!(updated.hour, 6);
        assert_eq!(updated.minute, 45);
        assert_eq!(updated.repeat_days, 0b0001_1111);
        assert!(updated.next_ring_at_ms.is_some());

        let mut state = updated.clone();
        alarm::set_enabled(&mut state, false, now).expect("disable alarm");
        save_alarm_state(&conn, &state).expect("save disabled alarm");
        let disabled = get_alarm(&conn, created.id).expect("get disabled alarm");
        assert!(!disabled.enabled);
        assert_eq!(disabled.next_ring_at_ms, None);

        alarm::set_enabled(&mut state, true, now).expect("enable alarm");
        alarm::snooze(&mut state, 5, now).expect("snooze alarm");
        save_alarm_state(&conn, &state).expect("save snoozed alarm");
        let snoozed = get_alarm(&conn, created.id).expect("get snoozed alarm");
        assert!(snoozed.enabled);
        assert_eq!(snoozed.snoozed_until_ms, Some(now + 5 * 60_000));
        assert_eq!(snoozed.next_ring_at_ms, Some(now + 5 * 60_000));

        alarm::dismiss(&mut state, now).expect("dismiss alarm");
        save_alarm_state(&conn, &state).expect("save dismissed alarm");
        let dismissed = get_alarm(&conn, created.id).expect("get dismissed alarm");
        assert!(dismissed.enabled);
        assert_eq!(dismissed.snoozed_until_ms, None);

        let second = create_alarm(&conn, Some("Tidur"), 22, 0, 0b0111_1111, now)
            .expect("create repeating alarm");
        assert_eq!(second.name, "Tidur");
        assert_eq!(second.position, 2);
        assert_eq!(second.repeat_days, 0b0111_1111);

        let all = list_alarms(&conn).expect("list alarms");
        assert_eq!(all.len(), 2);

        delete_alarm(&conn, created.id).expect("delete first alarm");
        assert_eq!(list_alarms(&conn).expect("list after delete").len(), 1);
        assert!(matches!(
            delete_alarm(&conn, created.id),
            Err(AppError::AlarmNotFound(id)) if id == created.id
        ));
        assert!(matches!(
            get_alarm(&conn, created.id),
            Err(AppError::AlarmNotFound(id)) if id == created.id
        ));
    }

    #[test]
    fn test_alarm_rearm_handles_missed_once_stale_repeat_and_snooze() {
        let conn = open_test_memory_db();
        let now = stopwatch::now_ms().expect("current time");

        let mut once = create_alarm(&conn, None, 7, 0, 0, now).expect("create once");
        once.next_ring_at_ms = Some(now - 60_000);
        save_alarm_state(&conn, &once).expect("save overdue once");

        let mut repeating =
            create_alarm(&conn, Some("Harian"), 6, 0, 0b0111_1111, now).expect("create repeating");
        repeating.next_ring_at_ms = Some(now - 24 * 3_600_000);
        save_alarm_state(&conn, &repeating).expect("save stale repeating");

        let mut snoozed =
            create_alarm(&conn, Some("Snooze"), 5, 0, 0, now).expect("create snoozed alarm");
        snoozed.snoozed_until_ms = Some(now - 1_000);
        snoozed.next_ring_at_ms = Some(now - 1_000);
        save_alarm_state(&conn, &snoozed).expect("save pending snooze");

        let changed = rearm_alarms(&conn, now).expect("rearm alarms");
        assert_eq!(changed, 2);

        let once_after = get_alarm(&conn, once.id).expect("get once after rearm");
        assert!(!once_after.enabled);
        assert_eq!(once_after.next_ring_at_ms, None);

        let repeating_after = get_alarm(&conn, repeating.id).expect("get repeating after rearm");
        assert!(repeating_after.enabled);
        assert!(
            repeating_after
                .next_ring_at_ms
                .is_some_and(|next| next > now)
        );

        let snoozed_after = get_alarm(&conn, snoozed.id).expect("get snoozed after rearm");
        assert!(snoozed_after.enabled);
        assert_eq!(snoozed_after.next_ring_at_ms, Some(now - 1_000));

        // Pemanggilan kedua harus idempotent.
        assert_eq!(rearm_alarms(&conn, now).expect("rearm again"), 0);
    }
}
