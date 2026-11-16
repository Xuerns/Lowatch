use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::AppError;
use crate::models::Stopwatch;

pub const MAX_NAME_CHARS: usize = 50;

/// Mengambil waktu saat ini dalam milidetik sejak Unix Epoch.
pub fn now_ms() -> Result<i64, AppError> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| AppError::SystemClockError)?;
    i64::try_from(duration.as_millis()).map_err(|_| AppError::SystemClockError)
}

/// Menghitung total waktu berjalan (`elapsed_ms`) secara aman terhadap pergeseran jam.
pub fn elapsed_ms(sw: &Stopwatch, now_ms: i64) -> i64 {
    if let Some(started_at) = sw.started_at_ms {
        let delta = now_ms.saturating_sub(started_at).max(0);
        sw.accumulated_ms.saturating_add(delta).max(0)
    } else {
        sw.accumulated_ms.max(0)
    }
}

/// Menjalankan stopwatch (bersifat idempotent jika sudah berjalan).
pub fn start(sw: &mut Stopwatch, now_ms: i64) {
    if sw.started_at_ms.is_none() {
        sw.started_at_ms = Some(now_ms);
        sw.heartbeat_ms = Some(now_ms);
        sw.updated_at_ms = now_ms;
    }
}

/// Menjeda stopwatch dan mengakumulasikan selisih waktu (bersifat idempotent jika sudah dijeda/idle).
pub fn pause(sw: &mut Stopwatch, now_ms: i64) {
    if let Some(started_at) = sw.started_at_ms.take() {
        let delta = now_ms.saturating_sub(started_at).max(0);
        sw.accumulated_ms = sw.accumulated_ms.saturating_add(delta).max(0);
        sw.heartbeat_ms = None;
        sw.updated_at_ms = now_ms;
    }
}

/// Mereset waktu stopwatch kembali ke 0 dan menghentikan status berjalan.
pub fn reset(sw: &mut Stopwatch, now_ms: i64) {
    sw.accumulated_ms = 0;
    sw.started_at_ms = None;
    sw.heartbeat_ms = None;
    sw.updated_at_ms = now_ms;
}

/// Memulihkan stopwatch yang masih berstatus running ketika aplikasi dibuka kembali setelah crash.
pub fn recover_from_crash(sw: &mut Stopwatch, now_ms: i64) {
    if let Some(started_at) = sw.started_at_ms.take() {
        let recovery_point = sw.heartbeat_ms.unwrap_or(started_at);
        let delta = recovery_point.saturating_sub(started_at).max(0);
        sw.accumulated_ms = sw.accumulated_ms.saturating_add(delta).max(0);
        sw.heartbeat_ms = None;
        sw.updated_at_ms = now_ms;
    }
}

/// Memvalidasi dan men-trim nama stopwatch (1..=50 karakter Unicode, tanpa karakter kontrol).
pub fn validate_name(raw: &str) -> Result<String, AppError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(AppError::EmptyName);
    }
    if trimmed.chars().any(|c| c.is_control()) {
        return Err(AppError::InvalidNameCharacters);
    }
    let count = trimmed.chars().count();
    if count > MAX_NAME_CHARS {
        return Err(AppError::NameTooLong(count));
    }
    Ok(trimmed.to_owned())
}

/// Menentukan nama stopwatch baru: memakai `"Stopwatch N"` bila input kosong/None, atau memvalidasi input.
pub fn resolve_create_name(raw: Option<&str>, default_index: i64) -> Result<String, AppError> {
    match raw {
        None => Ok(format!("Stopwatch {}", default_index.max(1))),
        Some(s) if s.trim().is_empty() => Ok(format!("Stopwatch {}", default_index.max(1))),
        Some(s) => validate_name(s),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::StopwatchStatus;

    fn sample_stopwatch() -> Stopwatch {
        Stopwatch {
            id: 1,
            name: "Stopwatch 1".to_owned(),
            accumulated_ms: 0,
            started_at_ms: None,
            heartbeat_ms: None,
            position: 1,
            created_at_ms: 1_000,
            updated_at_ms: 1_000,
        }
    }

    #[test]
    fn test_elapsed_on_idle_running_and_paused() {
        let mut sw = sample_stopwatch();
        assert_eq!(sw.status(), StopwatchStatus::Idle);
        assert_eq!(elapsed_ms(&sw, 2_000), 0);

        start(&mut sw, 2_000);
        assert_eq!(sw.status(), StopwatchStatus::Running);
        assert_eq!(elapsed_ms(&sw, 3_500), 1_500);

        pause(&mut sw, 3_500);
        assert_eq!(sw.status(), StopwatchStatus::Paused);
        assert_eq!(elapsed_ms(&sw, 9_999), 1_500);
    }

    #[test]
    fn test_start_pause_accumulate_multiple_cycles() {
        let mut sw = sample_stopwatch();

        start(&mut sw, 1_000);
        pause(&mut sw, 2_500);
        assert_eq!(sw.accumulated_ms, 1_500);
        assert_eq!(sw.started_at_ms, None);

        start(&mut sw, 5_000);
        assert_eq!(elapsed_ms(&sw, 5_800), 2_300);

        pause(&mut sw, 6_200);
        assert_eq!(sw.accumulated_ms, 2_700);
        assert_eq!(sw.started_at_ms, None);
        assert_eq!(sw.heartbeat_ms, None);
    }

    #[test]
    fn test_double_start_and_double_pause_are_idempotent() {
        let mut sw = sample_stopwatch();

        start(&mut sw, 1_000);
        start(&mut sw, 2_000);
        assert_eq!(sw.started_at_ms, Some(1_000));
        assert_eq!(sw.heartbeat_ms, Some(1_000));
        assert_eq!(sw.updated_at_ms, 1_000);

        pause(&mut sw, 3_000);
        assert_eq!(sw.accumulated_ms, 2_000);
        assert_eq!(sw.updated_at_ms, 3_000);

        pause(&mut sw, 9_000);
        assert_eq!(sw.accumulated_ms, 2_000);
        assert_eq!(sw.updated_at_ms, 3_000);
    }

    #[test]
    fn test_reset_from_running_and_paused() {
        let mut sw = sample_stopwatch();

        start(&mut sw, 1_000);
        reset(&mut sw, 2_500);
        assert_eq!(sw.accumulated_ms, 0);
        assert_eq!(sw.started_at_ms, None);
        assert_eq!(sw.heartbeat_ms, None);
        assert_eq!(sw.status(), StopwatchStatus::Idle);
        assert_eq!(sw.updated_at_ms, 2_500);

        start(&mut sw, 3_000);
        pause(&mut sw, 4_200);
        assert_eq!(sw.status(), StopwatchStatus::Paused);

        reset(&mut sw, 5_000);
        assert_eq!(sw.accumulated_ms, 0);
        assert_eq!(sw.started_at_ms, None);
        assert_eq!(sw.heartbeat_ms, None);
        assert_eq!(sw.status(), StopwatchStatus::Idle);
        assert_eq!(sw.updated_at_ms, 5_000);
    }

    #[test]
    fn test_clock_skew_never_produces_negative_delta() {
        let mut sw = sample_stopwatch();
        sw.accumulated_ms = 500;
        start(&mut sw, 10_000);

        assert_eq!(elapsed_ms(&sw, 8_000), 500);

        pause(&mut sw, 8_000);
        assert_eq!(sw.accumulated_ms, 500);
        assert_eq!(sw.started_at_ms, None);
    }

    #[test]
    fn test_crash_recovery_uses_heartbeat_ms() {
        let mut sw = sample_stopwatch();
        sw.accumulated_ms = 1_000;
        sw.started_at_ms = Some(10_000);
        sw.heartbeat_ms = Some(25_000);

        recover_from_crash(&mut sw, 90_000);
        assert_eq!(sw.accumulated_ms, 16_000);
        assert_eq!(sw.started_at_ms, None);
        assert_eq!(sw.heartbeat_ms, None);
        assert_eq!(sw.updated_at_ms, 90_000);
        assert_eq!(sw.status(), StopwatchStatus::Paused);
    }

    #[test]
    fn test_validate_name_rules() {
        assert_eq!(
            validate_name("  Belajar Rust  ").ok(),
            Some("Belajar Rust".to_owned())
        );

        assert!(matches!(validate_name(""), Err(AppError::EmptyName)));
        assert!(matches!(validate_name("   \t  "), Err(AppError::EmptyName)));

        let exact_50_unicode = "⏱".repeat(50);
        assert_eq!(
            validate_name(&exact_50_unicode).ok(),
            Some(exact_50_unicode)
        );

        let too_long_51_unicode = "⏱".repeat(51);
        assert!(matches!(
            validate_name(&too_long_51_unicode),
            Err(AppError::NameTooLong(51))
        ));

        assert!(matches!(
            validate_name("Halo\nDunia"),
            Err(AppError::InvalidNameCharacters)
        ));
        assert!(matches!(
            validate_name("Halo\0Dunia"),
            Err(AppError::InvalidNameCharacters)
        ));

        assert_eq!(
            resolve_create_name(None, 3).ok(),
            Some("Stopwatch 3".to_owned())
        );
        assert_eq!(
            resolve_create_name(Some("   "), 4).ok(),
            Some("Stopwatch 4".to_owned())
        );
        assert_eq!(
            resolve_create_name(Some("  Kerja  "), 4).ok(),
            Some("Kerja".to_owned())
        );
    }

    #[test]
    fn test_crash_recovery_without_heartbeat_and_with_clock_skew() {
        // Jika heartbeat_ms adalah None, recover_from_crash menggunakan started_at_ms (delta = 0)
        let mut sw = sample_stopwatch();
        sw.accumulated_ms = 2_500;
        sw.started_at_ms = Some(10_000);
        sw.heartbeat_ms = None;

        recover_from_crash(&mut sw, 50_000);
        assert_eq!(sw.accumulated_ms, 2_500);
        assert_eq!(sw.started_at_ms, None);
        assert_eq!(sw.heartbeat_ms, None);
        assert_eq!(sw.updated_at_ms, 50_000);
        assert_eq!(sw.status(), StopwatchStatus::Paused);

        // Jika heartbeat_ms < started_at_ms karena clock skew, delta tidak pernah negatif
        let mut sw_skew = sample_stopwatch();
        sw_skew.accumulated_ms = 1_200;
        sw_skew.started_at_ms = Some(20_000);
        sw_skew.heartbeat_ms = Some(15_000);

        recover_from_crash(&mut sw_skew, 60_000);
        assert_eq!(sw_skew.accumulated_ms, 1_200);
        assert_eq!(sw_skew.started_at_ms, None);
        assert_eq!(sw_skew.heartbeat_ms, None);
    }
}
