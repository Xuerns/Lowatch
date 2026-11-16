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
