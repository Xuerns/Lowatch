use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Stopwatch name cannot be empty.")]
    EmptyName,

    #[error("Stopwatch name must be at most 50 characters (currently {0} characters).")]
    NameTooLong(usize),

    #[error("Stopwatch name contains invalid characters.")]
    InvalidNameCharacters,

    #[error("Stopwatch with ID {0} not found.")]
    NotFound(i64),

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
