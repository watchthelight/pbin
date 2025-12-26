//! PBIN Core Library
//!
//! Provides format parsing, manifest handling, and target detection for PBIN files.

mod error;
mod header;
mod manifest;
mod target;

pub use error::{Error, Result};
pub use header::{PbinHeader, PAYLOAD_MARKER, PBIN_MAGIC, PBIN_VERSION};
pub use manifest::{Compression, PbinEntry, PbinManifest};
pub use target::Target;

/// Re-export blake3 for checksum verification.
pub use blake3;
