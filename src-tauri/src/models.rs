use serde::{Deserialize, Serialize};

/// Status turunan stopwatch (tidak disimpan di kolom database).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StopwatchStatus {
    Idle,
    Running,
    Paused,
}

/// Representasi baris tabel `stopwatches` di SQLite.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stopwatch {
    pub id: i64,
    pub name: String,
    pub accumulated_ms: i64,
    pub started_at_ms: Option<i64>,
    pub heartbeat_ms: Option<i64>,
    pub position: i64,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

/// Data Transfer Object (DTO) yang diserialisasi ke frontend melalui IPC Tauri.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StopwatchDto {
    pub id: i64,
    pub name: String,
    pub accumulated_ms: i64,
    pub started_at_ms: Option<i64>,
    pub status: StopwatchStatus,
    pub position: i64,
}

impl Stopwatch {
    /// Menentukan status berdasarkan state `started_at_ms` dan `accumulated_ms`.
    pub fn status(&self) -> StopwatchStatus {
        if self.started_at_ms.is_some() {
            StopwatchStatus::Running
        } else if self.accumulated_ms > 0 {
            StopwatchStatus::Paused
        } else {
            StopwatchStatus::Idle
        }
    }

    /// Mengonversi entity domain `Stopwatch` menjadi `StopwatchDto`.
    pub fn to_dto(&self) -> StopwatchDto {
        StopwatchDto {
            id: self.id,
            name: self.name.clone(),
            accumulated_ms: self.accumulated_ms,
            started_at_ms: self.started_at_ms,
            status: self.status(),
            position: self.position,
        }
    }
}

/// Status turunan timer countdown (tidak disimpan di kolom database).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TimerStatus {
    Idle,
    Running,
    Paused,
    Completed,
}

/// Representasi baris tabel `timers` di SQLite.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Timer {
    pub id: i64,
    pub name: String,
    pub duration_ms: i64,
    pub remaining_ms: i64,
    pub started_at_ms: Option<i64>,
    pub heartbeat_ms: Option<i64>,
    pub position: i64,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

/// Data Transfer Object (DTO) timer yang diserialisasi ke frontend melalui IPC Tauri.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimerDto {
    pub id: i64,
    pub name: String,
    pub duration_ms: i64,
    pub remaining_ms: i64,
    pub started_at_ms: Option<i64>,
    pub status: TimerStatus,
    pub position: i64,
}

impl Timer {
    /// Menentukan status timer berdasarkan `started_at_ms`, `remaining_ms`, dan `duration_ms`.
    pub fn status(&self) -> TimerStatus {
        if self.started_at_ms.is_some() {
            TimerStatus::Running
        } else if self.remaining_ms <= 0 {
            TimerStatus::Completed
        } else if self.remaining_ms < self.duration_ms {
            TimerStatus::Paused
        } else {
            TimerStatus::Idle
        }
    }

    /// Mengonversi entity domain `Timer` menjadi `TimerDto`.
    pub fn to_dto(&self) -> TimerDto {
        TimerDto {
            id: self.id,
            name: self.name.clone(),
            duration_ms: self.duration_ms,
            remaining_ms: self.remaining_ms,
            started_at_ms: self.started_at_ms,
            status: self.status(),
            position: self.position,
        }
    }
}

/// Representasi baris tabel `alarms` di SQLite.
///
/// `repeat_days` adalah bitmask Senin..Minggu (bit 0 = Senin). Nilai 0 berarti
/// alarm sekali pakai, bukan alarm yang tidak pernah berbunyi.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Alarm {
    pub id: i64,
    pub name: String,
    pub hour: u8,
    pub minute: u8,
    pub repeat_days: u8,
    pub enabled: bool,
    pub snoozed_until_ms: Option<i64>,
    pub next_ring_at_ms: Option<i64>,
    pub last_triggered_at_ms: Option<i64>,
    pub position: i64,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

/// Data Transfer Object alarm yang diserialisasi ke frontend melalui IPC Tauri.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlarmDto {
    pub id: i64,
    pub name: String,
    pub hour: u8,
    pub minute: u8,
    pub repeat_days: u8,
    pub enabled: bool,
    pub snoozed_until_ms: Option<i64>,
    pub next_ring_at_ms: Option<i64>,
    pub position: i64,
}

impl Alarm {
    /// Mengonversi entity domain `Alarm` menjadi `AlarmDto`.
    pub fn to_dto(&self) -> AlarmDto {
        AlarmDto {
            id: self.id,
            name: self.name.clone(),
            hour: self.hour,
            minute: self.minute,
            repeat_days: self.repeat_days,
            enabled: self.enabled,
            snoozed_until_ms: self.snoozed_until_ms,
            next_ring_at_ms: self.next_ring_at_ms,
            position: self.position,
        }
    }
}
