//! PBIN header structures and parsing.

use crate::{Compression, Error, Result};
use std::io::{Read, Write};

/// PBIN file magic bytes.
pub const PBIN_MAGIC: [u8; 4] = *b"PBIN";

/// Current format version.
pub const PBIN_VERSION: u16 = 1;

/// Header size in bytes.
pub const HEADER_SIZE: usize = 64;

/// Payload marker string.
pub const PAYLOAD_MARKER: &[u8] = b"__PBIN_PAYLOAD__";

/// The fixed 64-byte PBIN header.
#[derive(Debug, Clone)]
pub struct PbinHeader {
    /// Magic bytes (always "PBIN").
    pub magic: [u8; 4],
    /// Format version.
    pub version: u16,
    /// Compression algorithm.
    pub compression: Compression,
    /// Number of binary entries.
    pub entry_count: u8,
    /// Size of the JSON manifest.
    pub manifest_size: u32,
    /// Reserved flags.
    pub flags: u32,
}

impl PbinHeader {
    /// Creates a new header with default values.
    pub fn new(compression: Compression, entry_count: u8, manifest_size: u32) -> Self {
        Self {
            magic: PBIN_MAGIC,
            version: PBIN_VERSION,
            compression,
            entry_count,
            manifest_size,
            flags: 0,
        }
    }

    /// Reads a header from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < HEADER_SIZE {
            return Err(Error::HeaderTooShort {
                expected: HEADER_SIZE,
                actual: bytes.len(),
            });
        }

        let magic: [u8; 4] = bytes[0..4].try_into().unwrap();
        if magic != PBIN_MAGIC {
            return Err(Error::InvalidMagic(magic));
        }

        let version = u16::from_le_bytes(bytes[4..6].try_into().unwrap());
        if version != PBIN_VERSION {
            return Err(Error::UnsupportedVersion(version));
        }

        let compression = Compression::from_byte(bytes[6])?;
        let entry_count = bytes[7];
        let manifest_size = u32::from_le_bytes(bytes[8..12].try_into().unwrap());
        let flags = u32::from_le_bytes(bytes[12..16].try_into().unwrap());

        Ok(Self {
            magic,
            version,
            compression,
            entry_count,
            manifest_size,
            flags,
        })
    }

    /// Reads a header from a reader.
    pub fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        let mut bytes = [0u8; HEADER_SIZE];
        reader.read_exact(&mut bytes)?;
        Self::from_bytes(&bytes)
    }

    /// Writes the header to bytes.
    pub fn to_bytes(&self) -> [u8; HEADER_SIZE] {
        let mut bytes = [0u8; HEADER_SIZE];
        bytes[0..4].copy_from_slice(&self.magic);
        bytes[4..6].copy_from_slice(&self.version.to_le_bytes());
        bytes[6] = self.compression.as_byte();
        bytes[7] = self.entry_count;
        bytes[8..12].copy_from_slice(&self.manifest_size.to_le_bytes());
        bytes[12..16].copy_from_slice(&self.flags.to_le_bytes());
        // bytes[16..64] are reserved (zeros)
        bytes
    }

    /// Writes the header to a writer.
    pub fn write_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(&self.to_bytes())?;
        Ok(())
    }
}

/// Finds the payload marker in a byte slice and returns its offset.
pub fn find_payload_marker(data: &[u8]) -> Option<usize> {
    data.windows(PAYLOAD_MARKER.len())
        .position(|window| window == PAYLOAD_MARKER)
}
