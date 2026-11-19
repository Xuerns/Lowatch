use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Name cannot be empty.")]
    EmptyName,

    #[error("Name must be at most 50 characters (currently {0} characters).")]
    NameTooLong(usize),

    #[error("Name contains invalid characters.")]
    InvalidNameCharacters,

    #[error("Stopwatch with ID {0} not found.")]
    NotFound(i64),

    #[error("Timer with ID {0} not found.")]
    TimerNotFound(i64),

    #[error("Timer duration must be between 1 second and 99:59:59.")]
    InvalidTimerDuration,

    #[error("Alarm with ID {0} not found.")]
    AlarmNotFound(i64),

    #[error("Alarm time must be a valid time of day (00:00 to 23:59).")]
    InvalidAlarmTime,

    #[error("Alarm repeat days are invalid.")]
    InvalidAlarmRepeatDays,

    #[error("Snooze duration must be between 1 and 60 minutes.")]
    InvalidSnoozeDuration,

    #[error("Unable to compute the next alarm occurrence.")]
    AlarmSchedule,

    #[error("Failed to access internal state lock.")]
    StateLockPoisoned,

    #[error("System clock error: time is before Unix Epoch.")]
    SystemClockError,

    #[error(
        "Database file '{path}' is corrupted or unreadable ({reason}). Original file was preserved for data safety."
    )]
    DatabaseCorrupted { path: String, reason: String },

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("File I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Tauri error: {0}")]
    Tauri(#[from] tauri::Error),
}

impl From<AppError> for String {
    fn from(err: AppError) -> Self {
        err.to_string()
    }
}
