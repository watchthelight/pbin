//! PBIN manifest structures and serialization.

use crate::{Error, Result, Target};
use serde::{Deserialize, Serialize};

/// Compression algorithm used for payloads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Compression {
    /// No compression.
    None,
    /// Zstandard compression.
    Zstd,
    /// LZ4 compression.
    Lz4,
}

impl Compression {
    /// Returns the byte identifier for this compression type.
    pub fn as_byte(&self) -> u8 {
        match self {
            Compression::None => 0,
            Compression::Zstd => 1,
            Compression::Lz4 => 2,
        }
    }

    /// Parses a compression type from its byte identifier.
    pub fn from_byte(b: u8) -> Result<Self> {
        match b {
            0 => Ok(Compression::None),
            1 => Ok(Compression::Zstd),
            2 => Ok(Compression::Lz4),
            _ => Err(Error::UnknownCompression(b)),
        }
    }
}

impl Default for Compression {
    fn default() -> Self {
        Compression::Zstd
    }
}

/// An entry in the PBIN manifest representing one embedded binary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PbinEntry {
    /// Target platform identifier (e.g., "linux-x86_64").
    pub target: String,
    /// Byte offset from start of file to compressed data.
    pub offset: u64,
    /// Size of compressed data in bytes.
    pub compressed_size: u64,
    /// Size of uncompressed binary in bytes.
    pub uncompressed_size: u64,
    /// BLAKE3 checksum of uncompressed data (hex string).
    pub checksum: String,
}

impl PbinEntry {
    /// Creates a new entry.
    pub fn new(
        target: Target,
        offset: u64,
        compressed_size: u64,
        uncompressed_size: u64,
        checksum: [u8; 32],
    ) -> Self {
        Self {
            target: target.as_str().to_string(),
            offset,
            compressed_size,
            uncompressed_size,
            checksum: hex_encode(&checksum),
        }
    }

    /// Parses the target field.
    pub fn target(&self) -> Result<Target> {
        Target::from_str(&self.target).ok_or_else(|| Error::InvalidTarget(self.target.clone()))
    }

    /// Gets the checksum as bytes.
    pub fn checksum_bytes(&self) -> Result<[u8; 32]> {
        hex_decode(&self.checksum)
    }

    /// Verifies that the given data matches the checksum.
    pub fn verify_checksum(&self, data: &[u8]) -> Result<bool> {
        let expected = self.checksum_bytes()?;
        let actual = blake3::hash(data);
        Ok(actual.as_bytes() == &expected)
    }
}

/// The PBIN manifest containing metadata about all embedded binaries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PbinManifest {
    /// Application name.
    pub name: String,
    /// Application version.
    pub version: String,
    /// List of embedded binary entries.
    pub entries: Vec<PbinEntry>,
}

impl PbinManifest {
    /// Creates a new manifest.
    pub fn new(name: String, version: String) -> Self {
        Self {
            name,
            version,
            entries: Vec::new(),
        }
    }

    /// Adds an entry to the manifest.
    pub fn add_entry(&mut self, entry: PbinEntry) {
        self.entries.push(entry);
    }

    /// Finds an entry for the given target.
    pub fn find_entry(&self, target: Target) -> Option<&PbinEntry> {
        let target_str = target.as_str();
        self.entries.iter().find(|e| e.target == target_str)
    }

    /// Finds an entry for the current platform.
    pub fn find_current_entry(&self) -> Result<&PbinEntry> {
        let target = Target::detect_current().ok_or(Error::UnsupportedPlatform)?;
        self.find_entry(target)
            .ok_or_else(|| Error::TargetNotFound(target.as_str().to_string()))
    }

    /// Serializes the manifest to JSON.
    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }

    /// Serializes the manifest to pretty JSON.
    pub fn to_json_pretty(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Deserializes the manifest from JSON.
    pub fn from_json(json: &str) -> Result<Self> {
        Ok(serde_json::from_str(json)?)
    }

    /// Deserializes the manifest from JSON bytes.
    pub fn from_json_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(serde_json::from_slice(bytes)?)
    }
}

/// Encodes bytes to a hex string.
fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Decodes a hex string to bytes.
fn hex_decode(hex: &str) -> Result<[u8; 32]> {
    if hex.len() != 64 {
        return Err(Error::ChecksumMismatch {
            expected: "64 hex characters".to_string(),
            actual: format!("{} characters", hex.len()),
        });
    }

    let mut bytes = [0u8; 32];
    for (i, chunk) in hex.as_bytes().chunks(2).enumerate() {
        let s = std::str::from_utf8(chunk).map_err(|_| Error::ChecksumMismatch {
            expected: "valid hex".to_string(),
            actual: "invalid utf8".to_string(),
        })?;
        bytes[i] = u8::from_str_radix(s, 16).map_err(|_| Error::ChecksumMismatch {
            expected: "valid hex".to_string(),
            actual: format!("invalid hex: {}", s),
        })?;
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_roundtrip() {
        let bytes: [u8; 32] = [
            0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab,
            0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67,
            0x89, 0xab, 0xcd, 0xef,
        ];
        let hex = hex_encode(&bytes);
        let decoded = hex_decode(&hex).unwrap();
        assert_eq!(bytes, decoded);
    }

    #[test]
    fn test_manifest_json_roundtrip() {
        let mut manifest = PbinManifest::new("test".to_string(), "1.0.0".to_string());
        manifest.add_entry(PbinEntry::new(
            Target::LinuxX86_64,
            1000,
            500,
            1000,
            [0u8; 32],
        ));

        let json = manifest.to_json().unwrap();
        let parsed = PbinManifest::from_json(&json).unwrap();

        assert_eq!(parsed.name, manifest.name);
        assert_eq!(parsed.version, manifest.version);
        assert_eq!(parsed.entries.len(), 1);
        assert_eq!(parsed.entries[0].target, "linux-x86_64");
    }
}
