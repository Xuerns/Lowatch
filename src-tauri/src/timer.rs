use crate::error::AppError;
use crate::models::Timer;
use crate::stopwatch;

pub const DEFAULT_DURATION_MS: i64 = 5 * 60 * 1_000;
pub const MIN_DURATION_MS: i64 = 1_000;
pub const MAX_DURATION_MS: i64 = 99 * 3_600_000 + 59 * 60_000 + 59 * 1_000;

/// Memvalidasi durasi timer agar berada pada rentang 1 detik ..= 99:59:59.
pub fn validate_duration(duration_ms: i64) -> Result<i64, AppError> {
    if (MIN_DURATION_MS..=MAX_DURATION_MS).contains(&duration_ms) {
        Ok(duration_ms)
    } else {
        Err(AppError::InvalidTimerDuration)
    }
}

/// Menghitung sisa waktu timer (`remaining_ms`) secara aman terhadap pergeseran jam.
pub fn remaining_ms(timer: &Timer, now_ms: i64) -> i64 {
    if let Some(started_at) = timer.started_at_ms {
        let elapsed = now_ms.saturating_sub(started_at).max(0);
        timer.remaining_ms.saturating_sub(elapsed).max(0)
    } else {
        timer.remaining_ms.max(0)
    }
}

/// Menjalankan timer (idempotent jika sudah berjalan).
/// Jika timer sudah selesai (`remaining_ms <= 0`), dimulai ulang dari `duration_ms`.
pub fn start(timer: &mut Timer, now_ms: i64) {
    if timer.started_at_ms.is_none() {
        if timer.remaining_ms <= 0 {
            timer.remaining_ms = timer.duration_ms;
        }
        timer.started_at_ms = Some(now_ms);
        timer.heartbeat_ms = Some(now_ms);
        timer.updated_at_ms = now_ms;
    }
}

/// Menjeda timer dan mengurangi `remaining_ms` sesuai selisih waktu (idempotent jika tidak berjalan).
pub fn pause(timer: &mut Timer, now_ms: i64) {
    if let Some(started_at) = timer.started_at_ms.take() {
        let elapsed = now_ms.saturating_sub(started_at).max(0);
        timer.remaining_ms = timer.remaining_ms.saturating_sub(elapsed).max(0);
        timer.heartbeat_ms = None;
        timer.updated_at_ms = now_ms;
    }
}

/// Menandai timer selesai (`remaining_ms = 0`) dan menghentikan eksekusinya.
pub fn complete(timer: &mut Timer, now_ms: i64) {
    timer.remaining_ms = 0;
    timer.started_at_ms = None;
    timer.heartbeat_ms = None;
    timer.updated_at_ms = now_ms;
}

/// Mereset sisa waktu timer kembali ke `duration_ms` dan menghentikan status berjalan.
pub fn reset(timer: &mut Timer, now_ms: i64) {
    timer.remaining_ms = timer.duration_ms;
    timer.started_at_ms = None;
    timer.heartbeat_ms = None;
    timer.updated_at_ms = now_ms;
}

/// Mengubah durasi target timer dan mereset sisa waktu ke durasi baru tersebut.
pub fn set_duration(timer: &mut Timer, duration_ms: i64, now_ms: i64) -> Result<(), AppError> {
    let valid_ms = validate_duration(duration_ms)?;
    timer.duration_ms = valid_ms;
    timer.remaining_ms = valid_ms;
    timer.started_at_ms = None;
    timer.heartbeat_ms = None;
    timer.updated_at_ms = now_ms;
    Ok(())
}

/// Memulihkan timer yang masih berstatus running ketika aplikasi dibuka kembali setelah crash.
pub fn recover_from_crash(timer: &mut Timer, now_ms: i64) {
    if let Some(started_at) = timer.started_at_ms.take() {
        let recovery_point = timer.heartbeat_ms.unwrap_or(started_at);
        let elapsed = recovery_point.saturating_sub(started_at).max(0);
        timer.remaining_ms = timer.remaining_ms.saturating_sub(elapsed).max(0);
        timer.heartbeat_ms = None;
        timer.updated_at_ms = now_ms;
    }
}

/// Menentukan nama timer baru: memakai `"Timer N"` bila input kosong/None, atau memvalidasi input.
pub fn resolve_create_name(raw: Option<&str>, default_index: i64) -> Result<String, AppError> {
    match raw {
        None => Ok(format!("Timer {}", default_index.max(1))),
        Some(s) if s.trim().is_empty() => Ok(format!("Timer {}", default_index.max(1))),
        Some(s) => stopwatch::validate_name(s),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::TimerStatus;

    fn sample_timer() -> Timer {
        Timer {
            id: 1,
            name: "Timer 1".to_owned(),
            duration_ms: 60_000,
            remaining_ms: 60_000,
            started_at_ms: None,
            heartbeat_ms: None,
            position: 1,
            created_at_ms: 1_000,
            updated_at_ms: 1_000,
        }
    }

    #[test]
    fn test_timer_lifecycle_idle_running_paused_completed_reset() {
        let mut t = sample_timer();
        assert_eq!(t.status(), TimerStatus::Idle);
        assert_eq!(remaining_ms(&t, 2_000), 60_000);

        start(&mut t, 10_000);
        assert_eq!(t.status(), TimerStatus::Running);
        assert_eq!(remaining_ms(&t, 25_000), 45_000);

        pause(&mut t, 25_000);
        assert_eq!(t.status(), TimerStatus::Paused);
        assert_eq!(t.remaining_ms, 45_000);
        assert_eq!(remaining_ms(&t, 99_000), 45_000);

        start(&mut t, 30_000);
        assert_eq!(remaining_ms(&t, 75_000), 0);
        assert_eq!(remaining_ms(&t, 90_000), 0);

        complete(&mut t, 75_000);
        assert_eq!(t.status(), TimerStatus::Completed);
        assert_eq!(t.remaining_ms, 0);
        assert_eq!(t.started_at_ms, None);

        // Starting a completed timer restarts from duration_ms
        start(&mut t, 80_000);
        assert_eq!(t.status(), TimerStatus::Running);
        assert_eq!(t.remaining_ms, 60_000);

        reset(&mut t, 85_000);
        assert_eq!(t.status(), TimerStatus::Idle);
        assert_eq!(t.remaining_ms, 60_000);
        assert_eq!(t.started_at_ms, None);
    }

    #[test]
    fn test_validate_duration_and_set_duration() {
        assert!(matches!(validate_duration(0), Err(AppError::InvalidTimerDuration)));
        assert!(matches!(validate_duration(999), Err(AppError::InvalidTimerDuration)));
        assert!(matches!(
            validate_duration(MAX_DURATION_MS + 1),
            Err(AppError::InvalidTimerDuration)
        ));
        assert_eq!(validate_duration(1_000).ok(), Some(1_000));
        assert_eq!(validate_duration(MAX_DURATION_MS).ok(), Some(MAX_DURATION_MS));

        let mut t = sample_timer();
        start(&mut t, 1_000);
        set_duration(&mut t, 120_000, 5_000).expect("set_duration");
        assert_eq!(t.status(), TimerStatus::Idle);
        assert_eq!(t.duration_ms, 120_000);
        assert_eq!(t.remaining_ms, 120_000);
        assert_eq!(t.started_at_ms, None);
    }

    #[test]
    fn test_timer_crash_recovery_and_clock_skew() {
        let mut t = sample_timer();
        t.started_at_ms = Some(10_000);
        t.heartbeat_ms = Some(25_000);

        recover_from_crash(&mut t, 90_000);
        assert_eq!(t.remaining_ms, 45_000);
        assert_eq!(t.started_at_ms, None);
        assert_eq!(t.heartbeat_ms, None);
        assert_eq!(t.status(), TimerStatus::Paused);

        // Clock skew: now_ms < started_at_ms must never increase remaining_ms
        start(&mut t, 50_000);
        assert_eq!(remaining_ms(&t, 40_000), 45_000);
        pause(&mut t, 40_000);
        assert_eq!(t.remaining_ms, 45_000);
    }
}
