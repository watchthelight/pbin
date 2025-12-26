//! Binary segment analysis and deduplication.
//!
//! Parses ELF, Mach-O, and PE binaries to identify segments that can be
//! deduplicated across multiple binaries (e.g., identical data sections).

use crate::{CompressionError, Result};
use goblin::Object;
use std::collections::HashMap;

/// Represents a segment from a binary.
#[derive(Debug, Clone)]
pub struct Segment {
    /// Segment name (e.g., ".text", ".data", "__TEXT").
    pub name: String,
    /// Offset in the original binary.
    pub offset: usize,
    /// Size of the segment.
    pub size: usize,
    /// Whether this segment contains executable code.
    pub executable: bool,
    /// Hash of segment contents for deduplication.
    pub hash: [u8; 32],
}

/// Parsed binary with segment information.
#[derive(Debug)]
pub struct ParsedBinary {
    /// Target platform (e.g., "linux-x86_64").
    pub target: String,
    /// Detected architecture for BCJ filtering.
    pub arch: String,
    /// List of segments.
    pub segments: Vec<Segment>,
    /// Raw binary data.
    pub data: Vec<u8>,
}

impl ParsedBinary {
    /// Parse a binary and extract segment information.
    pub fn parse(target: &str, data: Vec<u8>) -> Result<Self> {
        let (segments, arch) = match Object::parse(&data) {
            Ok(Object::Elf(elf)) => parse_elf(&data, &elf),
            Ok(Object::Mach(mach)) => parse_mach(&data, &mach),
            Ok(Object::PE(pe)) => parse_pe(&data, &pe),
            Ok(_) => (Vec::new(), "unknown".to_string()),
            Err(e) => {
                return Err(CompressionError::Parse(format!(
                    "Failed to parse binary: {}",
                    e
                )))
            }
        };

        Ok(Self {
            target: target.to_string(),
            arch,
            segments,
            data,
        })
    }

    /// Get executable segments (for BCJ filtering).
    pub fn executable_segments(&self) -> Vec<&Segment> {
        self.segments.iter().filter(|s| s.executable).collect()
    }

    /// Get data for a specific segment.
    pub fn segment_data(&self, segment: &Segment) -> &[u8] {
        let end = (segment.offset + segment.size).min(self.data.len());
        &self.data[segment.offset..end]
    }
}

/// Parse ELF binary segments.
fn parse_elf(data: &[u8], elf: &goblin::elf::Elf) -> (Vec<Segment>, String) {
    let arch = match elf.header.e_machine {
        goblin::elf::header::EM_X86_64 => "x86_64",
        goblin::elf::header::EM_386 => "i686",
        goblin::elf::header::EM_AARCH64 => "aarch64",
        goblin::elf::header::EM_ARM => "arm",
        goblin::elf::header::EM_RISCV => "riscv64",
        goblin::elf::header::EM_PPC64 => "ppc64",
        _ => "unknown",
    }
    .to_string();

    let mut segments = Vec::new();

    for section in &elf.section_headers {
        if section.sh_size == 0 {
            continue;
        }

        let name = elf
            .shdr_strtab
            .get_at(section.sh_name)
            .unwrap_or("")
            .to_string();

        let offset = section.sh_offset as usize;
        let size = section.sh_size as usize;

        if offset + size > data.len() {
            continue;
        }

        let executable = section.sh_flags & goblin::elf::section_header::SHF_EXECINSTR as u64 != 0;
        let hash = blake3::hash(&data[offset..offset + size]).into();

        segments.push(Segment {
            name,
            offset,
            size,
            executable,
            hash,
        });
    }

    (segments, arch)
}

/// Parse Mach-O binary segments.
fn parse_mach(data: &[u8], mach: &goblin::mach::Mach) -> (Vec<Segment>, String) {
    match mach {
        goblin::mach::Mach::Binary(macho) => parse_macho_binary(data, macho),
        goblin::mach::Mach::Fat(fat) => {
            // For fat binaries, parse the first architecture
            if let Some(arch) = fat.iter_arches().next() {
                if let Ok(arch) = arch {
                    let start = arch.offset as usize;
                    let end = start + arch.size as usize;
                    if end <= data.len() {
                        let slice = &data[start..end];
                        if let Ok(Object::Mach(goblin::mach::Mach::Binary(macho))) =
                            Object::parse(slice)
                        {
                            return parse_macho_binary(slice, &macho);
                        }
                    }
                }
            }
            (Vec::new(), "unknown".to_string())
        }
    }
}

fn parse_macho_binary(
    data: &[u8],
    macho: &goblin::mach::MachO,
) -> (Vec<Segment>, String) {
    let arch = match macho.header.cputype() {
        goblin::mach::cputype::CPU_TYPE_X86_64 => "x86_64",
        goblin::mach::cputype::CPU_TYPE_ARM64 => "aarch64",
        goblin::mach::cputype::CPU_TYPE_ARM => "arm",
        _ => "unknown",
    }
    .to_string();

    let mut segments = Vec::new();

    for segment in &macho.segments {
        for (section, _) in segment.sections().unwrap_or_default() {
            let name = section.name().unwrap_or("").to_string();
            let offset = section.offset as usize;
            let size = section.size as usize;

            if offset + size > data.len() || size == 0 {
                continue;
            }

            // Check if section is executable (S_ATTR_PURE_INSTRUCTIONS or S_ATTR_SOME_INSTRUCTIONS)
            let executable = section.flags & 0x80000000 != 0 || section.flags & 0x400 != 0;
            let hash = blake3::hash(&data[offset..offset + size]).into();

            segments.push(Segment {
                name,
                offset,
                size,
                executable,
                hash,
            });
        }
    }

    (segments, arch)
}

/// Parse PE binary segments.
fn parse_pe(data: &[u8], pe: &goblin::pe::PE) -> (Vec<Segment>, String) {
    let arch = if pe.is_64 { "x86_64" } else { "i686" }.to_string();

    let mut segments = Vec::new();

    for section in &pe.sections {
        let name = section.name().unwrap_or("").to_string();
        let offset = section.pointer_to_raw_data as usize;
        let size = section.size_of_raw_data as usize;

        if offset + size > data.len() || size == 0 {
            continue;
        }

        // IMAGE_SCN_MEM_EXECUTE
        let executable = section.characteristics & 0x20000000 != 0;
        let hash = blake3::hash(&data[offset..offset + size]).into();

        segments.push(Segment {
            name,
            offset,
            size,
            executable,
            hash,
        });
    }

    (segments, arch)
}

/// Find duplicate segments across multiple binaries.
pub fn find_duplicates(binaries: &[ParsedBinary]) -> HashMap<[u8; 32], Vec<(usize, usize)>> {
    let mut hash_map: HashMap<[u8; 32], Vec<(usize, usize)>> = HashMap::new();

    for (bin_idx, binary) in binaries.iter().enumerate() {
        for (seg_idx, segment) in binary.segments.iter().enumerate() {
            hash_map
                .entry(segment.hash)
                .or_default()
                .push((bin_idx, seg_idx));
        }
    }

    // Keep only hashes that appear multiple times
    hash_map.retain(|_, v| v.len() > 1);

    hash_map
}

/// Calculate potential savings from segment deduplication.
pub fn estimate_savings(binaries: &[ParsedBinary]) -> usize {
    let duplicates = find_duplicates(binaries);
    let mut savings = 0;

    for (_hash, locations) in duplicates {
        if locations.len() > 1 {
            // First occurrence is kept, rest are deduplicated
            for (bin_idx, seg_idx) in locations.iter().skip(1) {
                if let Some(segment) = binaries.get(*bin_idx).and_then(|b| b.segments.get(*seg_idx))
                {
                    savings += segment.size;
                }
            }
        }
    }

    savings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_segment_hash() {
        let data1 = vec![1, 2, 3, 4, 5];
        let data2 = vec![1, 2, 3, 4, 5];
        let data3 = vec![1, 2, 3, 4, 6];

        let hash1: [u8; 32] = blake3::hash(&data1).into();
        let hash2: [u8; 32] = blake3::hash(&data2).into();
        let hash3: [u8; 32] = blake3::hash(&data3).into();

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_find_duplicates() {
        // Create mock binaries with some duplicate segments
        let binaries = vec![
            ParsedBinary {
                target: "linux-x86_64".to_string(),
                arch: "x86_64".to_string(),
                segments: vec![
                    Segment {
                        name: ".text".to_string(),
                        offset: 0,
                        size: 100,
                        executable: true,
                        hash: [1; 32],
                    },
                    Segment {
                        name: ".data".to_string(),
                        offset: 100,
                        size: 50,
                        executable: false,
                        hash: [2; 32], // Same as darwin
                    },
                ],
                data: vec![0; 150],
            },
            ParsedBinary {
                target: "darwin-x86_64".to_string(),
                arch: "x86_64".to_string(),
                segments: vec![
                    Segment {
                        name: "__TEXT".to_string(),
                        offset: 0,
                        size: 100,
                        executable: true,
                        hash: [3; 32], // Different
                    },
                    Segment {
                        name: "__DATA".to_string(),
                        offset: 100,
                        size: 50,
                        executable: false,
                        hash: [2; 32], // Same as linux
                    },
                ],
                data: vec![0; 150],
            },
        ];

        let duplicates = find_duplicates(&binaries);

        // Should find one duplicate (the .data/__DATA segment with hash [2; 32])
        assert_eq!(duplicates.len(), 1);
        assert!(duplicates.contains_key(&[2; 32]));
    }
}
