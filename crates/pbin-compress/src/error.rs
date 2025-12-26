//! Error types for compression operations.

use thiserror::Error;

/// Result type for compression operations.
pub type Result<T> = std::result::Result<T, CompressionError>;

/// Errors that can occur during compression.
#[derive(Error, Debug)]
pub enum CompressionError {
    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Zstd compression error.
    #[error("Zstd error: {0}")]
    Zstd(String),

    /// Delta compression error.
    #[error("Delta compression error: {0}")]
    Delta(String),

    /// Binary parsing error.
    #[error("Binary parsing error: {0}")]
    Parse(String),

    /// Invalid data.
    #[error("Invalid data: {0}")]
    InvalidData(String),

    /// Decompression error.
    #[error("Decompression error: {0}")]
    Decompression(String),
}
