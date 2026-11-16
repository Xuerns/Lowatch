use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Nama stopwatch tidak boleh kosong.")]
    EmptyName,

    #[error("Nama stopwatch maksimal 50 karakter (saat ini {0} karakter).")]
    NameTooLong(usize),

    #[error("Nama stopwatch mengandung karakter tidak valid.")]
    InvalidNameCharacters,

    #[error("Stopwatch dengan ID {0} tidak ditemukan.")]
    NotFound(i64),

    #[error("Gagal mengakses pengunci state internal.")]
    StateLockPoisoned,

    #[error("Kesalahan jam sistem: waktu berada sebelum Unix Epoch.")]
    SystemClockError,

    #[error(
        "Berkas database '{path}' rusak atau tidak dapat dibaca ({reason}). Berkas asli tidak ditimpa demi keamanan data."
    )]
    DatabaseCorrupted { path: String, reason: String },

    #[error("Kesalahan database: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Kesalahan I/O berkas: {0}")]
    Io(#[from] std::io::Error),

    #[error("Kesalahan Tauri: {0}")]
    Tauri(#[from] tauri::Error),
}

impl From<AppError> for String {
    fn from(err: AppError) -> Self {
        err.to_string()
    }
}
