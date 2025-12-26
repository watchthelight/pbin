//! PBIN Compression Pipeline
//!
//! Provides advanced compression for PBIN files using:
//! - BCJ filters for x86/ARM code preprocessing
//! - Delta compression for similar binaries
//! - Zstd dictionary training
//! - Segment deduplication

pub mod bcj;
pub mod delta;
pub mod dict;
pub mod pipeline;
pub mod segment;

mod error;

pub use error::{CompressionError, Result};
pub use pipeline::{CompressionLevel, CompressionPipeline, PlatformTier};
