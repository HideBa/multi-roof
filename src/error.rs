use thiserror::Error;

/// Custom error types for the multi-roof-assignment project
#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Command line argument error: {0}")]
    CommandLine(#[from] clap::error::Error),

    #[error("Rerun error: {0}")]
    Rerun(#[from] rerun::RecordingStreamError),
}

/// Type alias for Result with Error
pub type Result<T> = std::result::Result<T, Error>;
