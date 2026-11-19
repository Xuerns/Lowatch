use chrono::{
    Datelike, Duration as ChronoDuration, Local, LocalResult, NaiveDateTime, TimeZone, Timelike,
    Weekday,
};

use crate::error::AppError;
use crate::models::Alarm;
use crate::stopwatch;

/// Snooze default dan batas aman agar pengguna tidak menggeser alarm terlalu jauh.
pub const DEFAULT_SNOOZE_MINUTES: i64 = 5;
pub const MAX_SNOOZE_MINUTES: i64 = 60;

/// Bit 0 = Senin, bit 6 = Minggu. Nilai 0 berarti alarm sekali pakai.
pub const MAX_REPEAT_DAYS_MASK: u8 = 0b0111_1111;

/// Rentang pencarian hari untuk menemukan jadwal berikutnya, termasuk satu
/// siklus mingguan penuh plus toleransi DST.
const SEARCH_DAYS: i64 = 15;

/// Memvalidasi komponen jam (0..=23).
pub fn validate_hour(hour: i64) -> Result<u8, AppError> {
    if (0..=23).contains(&hour) {
        u8::try_from(hour).map_err(|_| AppError::InvalidAlarmTime)
    } else {
        Err(AppError::InvalidAlarmTime)
    }
}

/// Memvalidasi komponen menit (0..=59).
pub fn validate_minute(minute: i64) -> Result<u8, AppError> {
    if (0..=59).contains(&minute) {
        u8::try_from(minute).map_err(|_| AppError::InvalidAlarmTime)
    } else {
        Err(AppError::InvalidAlarmTime)
    }
}

/// Memvalidasi bitmask hari pengulangan.
pub fn validate_repeat_days(mask: i64) -> Result<u8, AppError> {
    if (0..=i64::from(MAX_REPEAT_DAYS_MASK)).contains(&mask) {
        u8::try_from(mask).map_err(|_| AppError::InvalidAlarmRepeatDays)
    } else {
        Err(AppError::InvalidAlarmRepeatDays)
    }
}

/// Memvalidasi durasi snooze dalam menit.
pub fn validate_snooze_minutes(minutes: i64) -> Result<i64, AppError> {
    if (1..=MAX_SNOOZE_MINUTES).contains(&minutes) {
        Ok(minutes)
    } else {
        Err(AppError::InvalidSnoozeDuration)
    }
}

/// Menentukan jam alarm baru: jam berikutnya pada menit ke-0.
pub fn next_hour_boundary(now_ms: i64) -> Result<(u8, u8), AppError> {
    let local = local_from_ms(now_ms)?;
    let next_hour = u8::try_from((local.hour() + 1) % 24).map_err(|_| AppError::AlarmSchedule)?;
    Ok((next_hour, 0))
}

/// Menyelesaikan waktu alarm saat pembuatan: memakai input pengguna jika lengkap,
/// atau jam berikutnya jika keduanya kosong.
pub fn resolve_create_time(
    hour: Option<i64>,
    minute: Option<i64>,
    now_ms: i64,
) -> Result<(u8, u8), AppError> {
    match (hour, minute) {
        (Some(raw_hour), Some(raw_minute)) => {
            Ok((validate_hour(raw_hour)?, validate_minute(raw_minute)?))
        }
        (None, None) => next_hour_boundary(now_ms),
        _ => Err(AppError::InvalidAlarmTime),
    }
}

/// Menentukan nama alarm baru: `"Alarm N"` bila kosong, atau nama tervalidasi.
pub fn resolve_create_name(raw: Option<&str>, default_index: i64) -> Result<String, AppError> {
    match raw {
        None => Ok(format!("Alarm {}", default_index.max(1))),
        Some(s) if s.trim().is_empty() => Ok(format!("Alarm {}", default_index.max(1))),
        Some(s) => stopwatch::validate_name(s),
    }
}

/// Mengubah hari kalender menjadi bitmask Senin..Minggu.
pub fn weekday_bit(weekday: Weekday) -> u8 {
    1_u8 << weekday.num_days_from_monday()
}

/// Menghitung kemunculan lokal berikutnya yang cocok dengan jadwal alarm.
///
/// `repeat_days == 0` berarti alarm sekali pakai: fungsi tetap memilih
/// kemunculan berikutnya (hari ini bila belum lewat, atau besok).
pub fn next_occurrence_naive(
    hour: u8,
    minute: u8,
    repeat_days: u8,
    now_local: NaiveDateTime,
) -> Option<NaiveDateTime> {
    for offset in 0..=SEARCH_DAYS {
        let date = now_local
            .date()
            .checked_add_signed(ChronoDuration::days(offset))?;
        if repeat_days != 0 && (repeat_days & weekday_bit(date.weekday())) == 0 {
            continue;
        }
        let candidate = date.and_hms_opt(u32::from(hour), u32::from(minute), 0)?;
        if candidate > now_local {
            return Some(candidate);
        }
    }
    None
}

/// Menghitung timestamp UTC untuk kemunculan terjadwal berikutnya.
///
/// Kandidat yang jatuh pada celah DST dilewati, lalu pencarian berlanjut ke
/// hari berikutnya yang cocok.
pub fn compute_regular_next_ring_at_ms(
    hour: u8,
    minute: u8,
    repeat_days: u8,
    now_ms: i64,
) -> Result<Option<i64>, AppError> {
    let now_local = local_from_ms(now_ms)?;
    for offset in 0..=SEARCH_DAYS {
        let date = now_local
            .date()
            .checked_add_signed(ChronoDuration::days(offset));
        let Some(date) = date else {
            break;
        };
        if repeat_days != 0 && (repeat_days & weekday_bit(date.weekday())) == 0 {
            continue;
        }
        let Some(candidate) = date.and_hms_opt(u32::from(hour), u32::from(minute), 0) else {
            continue;
        };
        if candidate <= now_local {
            continue;
        }
        if let Some(timestamp_ms) = resolve_local_candidate(candidate) {
            return Ok(Some(timestamp_ms));
        }
    }
    Ok(None)
}

/// Menghitung waktu berbunyi berikutnya dari state alarm saat ini.
/// Snooze selalu diprioritaskan di atas jadwal berulang.
pub fn compute_next_ring_at_ms(alarm: &Alarm, now_ms: i64) -> Result<Option<i64>, AppError> {
    if !alarm.enabled {
        return Ok(None);
    }
    if let Some(snoozed_until) = alarm.snoozed_until_ms {
        return Ok(Some(snoozed_until));
    }
    compute_regular_next_ring_at_ms(alarm.hour, alarm.minute, alarm.repeat_days, now_ms)
}

/// Menyegarkan `next_ring_at_ms` agar sesuai dengan state alarm.
pub fn schedule(alarm: &mut Alarm, now_ms: i64) -> Result<(), AppError> {
    alarm.next_ring_at_ms = compute_next_ring_at_ms(alarm, now_ms)?;
    Ok(())
}

/// Menandai alarm sedang berbunyi.
///
/// Alarm sekali pakai langsung dinonaktifkan. Alarm berulang tetap aktif dan
/// langsung dijadwalkan ke kemunculan berikutnya.
pub fn start_ringing(alarm: &mut Alarm, now_ms: i64) -> Result<(), AppError> {
    alarm.snoozed_until_ms = None;
    alarm.last_triggered_at_ms = Some(now_ms);
    alarm.updated_at_ms = now_ms;
    alarm.enabled = alarm.repeat_days != 0;
    alarm.next_ring_at_ms = if alarm.enabled {
        compute_regular_next_ring_at_ms(alarm.hour, alarm.minute, alarm.repeat_days, now_ms)?
    } else {
        None
    };
    Ok(())
}

/// Menunda alarm selama `minutes` menit dan memastikan alarm aktif kembali.
pub fn snooze(alarm: &mut Alarm, minutes: i64, now_ms: i64) -> Result<(), AppError> {
    let valid_minutes = validate_snooze_minutes(minutes)?;
    let snoozed_until = now_ms.saturating_add(valid_minutes.saturating_mul(60_000));
    alarm.enabled = true;
    alarm.snoozed_until_ms = Some(snoozed_until);
    alarm.next_ring_at_ms = Some(snoozed_until);
    alarm.last_triggered_at_ms = Some(now_ms);
    alarm.updated_at_ms = now_ms;
    Ok(())
}

/// Menutup alarm yang sedang berbunyi.
///
/// Snooze yang tertunda dibatalkan. Alarm sekali pakai menjadi nonaktif,
/// sedangkan alarm berulang dijadwalkan ke kemunculan berikutnya.
pub fn dismiss(alarm: &mut Alarm, now_ms: i64) -> Result<(), AppError> {
    alarm.snoozed_until_ms = None;
    alarm.updated_at_ms = now_ms;
    if alarm.repeat_days == 0 {
        alarm.enabled = false;
        alarm.next_ring_at_ms = None;
    } else if alarm.enabled {
        alarm.next_ring_at_ms =
            compute_regular_next_ring_at_ms(alarm.hour, alarm.minute, alarm.repeat_days, now_ms)?;
    } else {
        alarm.next_ring_at_ms = None;
    }
    Ok(())
}

/// Mengaktifkan atau menonaktifkan alarm.
///
/// Menonaktifkan alarm juga membatalkan snooze agar alarm lama tidak berbunyi
/// tiba-tiba ketika diaktifkan kembali.
pub fn set_enabled(alarm: &mut Alarm, enabled: bool, now_ms: i64) -> Result<(), AppError> {
    alarm.enabled = enabled;
    alarm.snoozed_until_ms = None;
    alarm.updated_at_ms = now_ms;
    alarm.next_ring_at_ms = if enabled {
        compute_regular_next_ring_at_ms(alarm.hour, alarm.minute, alarm.repeat_days, now_ms)?
    } else {
        None
    };
    Ok(())
}

fn local_from_ms(now_ms: i64) -> Result<NaiveDateTime, AppError> {
    Local
        .timestamp_millis_opt(now_ms)
        .single()
        .map(|datetime| datetime.naive_local())
        .ok_or(AppError::AlarmSchedule)
}

fn resolve_local_candidate(naive: NaiveDateTime) -> Option<i64> {
    match Local.from_local_datetime(&naive) {
        LocalResult::Single(datetime) => Some(datetime.timestamp_millis()),
        LocalResult::Ambiguous(earlier, _) => Some(earlier.timestamp_millis()),
        LocalResult::None => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn sample_alarm() -> Alarm {
        Alarm {
            id: 1,
            name: "Alarm 1".to_owned(),
            hour: 7,
            minute: 30,
            repeat_days: 0,
            enabled: true,
            snoozed_until_ms: None,
            next_ring_at_ms: None,
            last_triggered_at_ms: None,
            position: 1,
            created_at_ms: 1_000,
            updated_at_ms: 1_000,
        }
    }

    fn local_datetime(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(year, month, day)
            .expect("valid date")
            .and_hms_opt(hour, minute, 0)
            .expect("valid time")
    }

    #[test]
    fn test_validate_time_repeat_days_and_snooze() {
        assert_eq!(validate_hour(0).ok(), Some(0));
        assert_eq!(validate_hour(23).ok(), Some(23));
        assert!(matches!(validate_hour(-1), Err(AppError::InvalidAlarmTime)));
        assert!(matches!(validate_hour(24), Err(AppError::InvalidAlarmTime)));

        assert_eq!(validate_minute(0).ok(), Some(0));
        assert_eq!(validate_minute(59).ok(), Some(59));
        assert!(matches!(
            validate_minute(60),
            Err(AppError::InvalidAlarmTime)
        ));

        assert_eq!(validate_repeat_days(0).ok(), Some(0));
        assert_eq!(validate_repeat_days(127).ok(), Some(MAX_REPEAT_DAYS_MASK));
        assert!(matches!(
            validate_repeat_days(128),
            Err(AppError::InvalidAlarmRepeatDays)
        ));
        assert!(matches!(
            validate_repeat_days(-1),
            Err(AppError::InvalidAlarmRepeatDays)
        ));

        assert_eq!(
            resolve_create_time(Some(6), Some(45), 1_000).ok(),
            Some((6, 45))
        );
        assert!(matches!(
            resolve_create_time(Some(6), None, 1_000),
            Err(AppError::InvalidAlarmTime)
        ));
        assert!(matches!(
            resolve_create_time(None, None, i64::MAX),
            Err(AppError::AlarmSchedule)
        ));

        assert!(matches!(
            validate_snooze_minutes(0),
            Err(AppError::InvalidSnoozeDuration)
        ));
        assert!(matches!(
            validate_snooze_minutes(61),
            Err(AppError::InvalidSnoozeDuration)
        ));
        assert_eq!(validate_snooze_minutes(10).ok(), Some(10));
    }

    #[test]
    fn test_weekday_bit_mapping_starts_on_monday() {
        assert_eq!(weekday_bit(Weekday::Mon), 1);
        assert_eq!(weekday_bit(Weekday::Tue), 2);
        assert_eq!(weekday_bit(Weekday::Wed), 4);
        assert_eq!(weekday_bit(Weekday::Thu), 8);
        assert_eq!(weekday_bit(Weekday::Fri), 16);
        assert_eq!(weekday_bit(Weekday::Sat), 32);
        assert_eq!(weekday_bit(Weekday::Sun), 64);
    }

    #[test]
    fn test_next_occurrence_once_and_daily() {
        // Senin, 16 November 2026, 08:00.
        let now = local_datetime(2026, 11, 16, 8, 0);

        assert_eq!(
            next_occurrence_naive(7, 30, 0, now),
            Some(local_datetime(2026, 11, 17, 7, 30))
        );
        assert_eq!(
            next_occurrence_naive(9, 0, 0, now),
            Some(local_datetime(2026, 11, 16, 9, 0))
        );
        assert_eq!(
            next_occurrence_naive(8, 0, 0, now),
            Some(local_datetime(2026, 11, 17, 8, 0))
        );
        assert_eq!(
            next_occurrence_naive(7, 30, MAX_REPEAT_DAYS_MASK, now),
            Some(local_datetime(2026, 11, 17, 7, 30))
        );
    }

    #[test]
    fn test_next_occurrence_respects_repeat_days() {
        // Jumat, 20 November 2026, 08:00. Alarm pukul 07:30 sudah lewat.
        let friday = local_datetime(2026, 11, 20, 8, 0);
        let weekdays = 0b0001_1111;
        assert_eq!(
            next_occurrence_naive(7, 30, weekdays, friday),
            Some(local_datetime(2026, 11, 23, 7, 30))
        );

        // Hanya Rabu, dari Senin.
        let monday = local_datetime(2026, 11, 16, 8, 0);
        let wednesday_only = weekday_bit(Weekday::Wed);
        assert_eq!(
            next_occurrence_naive(7, 30, wednesday_only, monday),
            Some(local_datetime(2026, 11, 18, 7, 30))
        );

        // Akhir pekan, dari Jumat malam.
        let weekend = weekday_bit(Weekday::Sat) | weekday_bit(Weekday::Sun);
        assert_eq!(
            next_occurrence_naive(7, 30, weekend, friday),
            Some(local_datetime(2026, 11, 21, 7, 30))
        );

        // Senin pagi sebelum waktunya: tetap hari ini.
        let monday_early = local_datetime(2026, 11, 16, 6, 0);
        assert_eq!(
            next_occurrence_naive(7, 30, weekday_bit(Weekday::Mon), monday_early),
            Some(local_datetime(2026, 11, 16, 7, 30))
        );
    }

    #[test]
    fn test_snooze_takes_priority_over_regular_schedule() {
        let now_ms = 1_700_000_000_000;
        let mut alarm = sample_alarm();
        alarm.snoozed_until_ms = Some(now_ms + 120_000);

        assert_eq!(
            compute_next_ring_at_ms(&alarm, now_ms).ok(),
            Some(Some(now_ms + 120_000))
        );

        alarm.enabled = false;
        assert_eq!(compute_next_ring_at_ms(&alarm, now_ms).ok(), Some(None));
    }

    #[test]
    fn test_ring_snooze_and_dismiss_lifecycle() {
        let now_ms = stopwatch::now_ms().expect("current time");

        let mut once = sample_alarm();
        assert_eq!(once.repeat_days, 0);
        start_ringing(&mut once, now_ms).expect("ring one-shot alarm");
        assert!(!once.enabled);
        assert_eq!(once.next_ring_at_ms, None);
        assert_eq!(once.snoozed_until_ms, None);
        assert_eq!(once.last_triggered_at_ms, Some(now_ms));

        snooze(&mut once, DEFAULT_SNOOZE_MINUTES, now_ms).expect("snooze one-shot alarm");
        assert!(once.enabled);
        assert_eq!(
            once.next_ring_at_ms,
            Some(now_ms + DEFAULT_SNOOZE_MINUTES * 60_000)
        );
        assert_eq!(
            once.snoozed_until_ms,
            Some(now_ms + DEFAULT_SNOOZE_MINUTES * 60_000)
        );

        dismiss(&mut once, now_ms).expect("dismiss one-shot alarm");
        assert!(!once.enabled);
        assert_eq!(once.snoozed_until_ms, None);
        assert_eq!(once.next_ring_at_ms, None);

        let mut repeating = sample_alarm();
        repeating.repeat_days = MAX_REPEAT_DAYS_MASK;
        start_ringing(&mut repeating, now_ms).expect("ring repeating alarm");
        assert!(repeating.enabled);
        assert!(repeating.next_ring_at_ms.is_some_and(|next| next > now_ms));

        set_enabled(&mut repeating, false, now_ms).expect("disable repeating alarm");
        assert!(!repeating.enabled);
        assert_eq!(repeating.next_ring_at_ms, None);

        set_enabled(&mut repeating, true, now_ms).expect("enable repeating alarm");
        assert!(repeating.enabled);
        assert!(repeating.next_ring_at_ms.is_some());
    }

    #[test]
    fn test_resolve_create_name() {
        assert_eq!(
            resolve_create_name(None, 2).ok(),
            Some("Alarm 2".to_owned())
        );
        assert_eq!(
            resolve_create_name(Some("   "), 3).ok(),
            Some("Alarm 3".to_owned())
        );
        assert_eq!(
            resolve_create_name(Some("  Bangun  "), 3).ok(),
            Some("Bangun".to_owned())
        );
        assert!(matches!(
            resolve_create_name(Some("Alarm\nPagi"), 1),
            Err(AppError::InvalidNameCharacters)
        ));
    }
}
