//! Error types for PBIN operations.

use thiserror::Error;

/// Result type for PBIN operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during PBIN operations.
#[derive(Error, Debug)]
pub enum Error {
    /// Invalid magic bytes in header.
    #[error("invalid magic bytes: expected 'PBIN', got {0:?}")]
    InvalidMagic([u8; 4]),

    /// Unsupported format version.
    #[error("unsupported version: {0}")]
    UnsupportedVersion(u16),

    /// Unknown compression type.
    #[error("unknown compression type: {0}")]
    UnknownCompression(u8),

    /// Invalid target string.
    #[error("invalid target: {0}")]
    InvalidTarget(String),

    /// Target not found in manifest.
    #[error("target not found in manifest: {0}")]
    TargetNotFound(String),

    /// Payload marker not found.
    #[error("payload marker '__PBIN_PAYLOAD__' not found")]
    PayloadMarkerNotFound,

    /// Checksum mismatch.
    #[error("checksum mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch { expected: String, actual: String },

    /// Header too short.
    #[error("header too short: expected at least {expected} bytes, got {actual}")]
    HeaderTooShort { expected: usize, actual: usize },

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON parsing error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Current platform not supported.
    #[error("current platform is not supported")]
    UnsupportedPlatform,
}
