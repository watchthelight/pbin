//! Compression pipeline orchestration.
//!
//! Coordinates BCJ filtering, delta compression, dictionary training,
//! and final zstd compression for optimal results.

use crate::bcj::{BcjArch, BcjFilter};
use crate::delta::{self, DeltaGroup};
use crate::dict::{self, TrainedDictionary, DEFAULT_DICT_SIZE};
use crate::{CompressionError, Result};
use std::collections::HashMap;

/// Platform tier classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformTier {
    /// Core platforms (most common, highest priority).
    /// Linux/macOS/Windows on x86_64 and ARM64.
    Core,
    /// Standard platforms (common, good support).
    /// Adds more Linux architectures, musl variants.
    Standard,
    /// Extended platforms (specialized, full support).
    /// Adds BSDs, Android, iOS, embedded targets.
    Extended,
}

impl PlatformTier {
    /// Get targets for this tier.
    pub fn targets(&self) -> Vec<&'static str> {
        match self {
            PlatformTier::Core => vec![
                "linux-x86_64",
                "linux-aarch64",
                "darwin-x86_64",
                "darwin-aarch64",
                "windows-x86_64",
                "windows-aarch64",
            ],
            PlatformTier::Standard => {
                let mut targets = PlatformTier::Core.targets();
                targets.extend(vec![
                    "linux-x86_64-musl",
                    "linux-aarch64-musl",
                    "linux-armv7",
                    "linux-riscv64",
                    "linux-ppc64le",
                    "linux-s390x",
                    "windows-x86",
                ]);
                targets
            }
            PlatformTier::Extended => {
                let mut targets = PlatformTier::Standard.targets();
                targets.extend(vec![
                    "freebsd-x86_64",
                    "freebsd-aarch64",
                    "netbsd-x86_64",
                    "openbsd-x86_64",
                    "android-aarch64",
                    "android-armv7",
                    "android-x86_64",
                    "ios-aarch64",
                    "linux-mips64",
                    "linux-loongarch64",
                    "wasi-wasm32",
                ]);
                targets
            }
        }
    }
}

/// Compression level presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionLevel {
    /// Fast compression, larger output.
    Fast,
    /// Balanced compression (default).
    Balanced,
    /// Maximum compression, slower.
    Maximum,
}

impl CompressionLevel {
    /// Get zstd compression level.
    pub fn zstd_level(&self) -> i32 {
        match self {
            CompressionLevel::Fast => 3,
            CompressionLevel::Balanced => 12,
            CompressionLevel::Maximum => 19,
        }
    }

    /// Get similarity threshold for delta compression.
    pub fn delta_threshold(&self) -> f64 {
        match self {
            CompressionLevel::Fast => 0.8,     // Only very similar binaries
            CompressionLevel::Balanced => 0.6, // Moderately similar
            CompressionLevel::Maximum => 0.4,  // More aggressive grouping
        }
    }
}

/// Compressed binary entry.
#[derive(Debug)]
pub struct CompressedEntry {
    /// Target platform.
    pub target: String,
    /// Compressed data.
    pub data: Vec<u8>,
    /// Whether BCJ filter was applied.
    pub bcj_filtered: bool,
    /// If stored as delta, reference target.
    pub delta_reference: Option<String>,
    /// Original uncompressed size.
    pub original_size: usize,
}

/// Compression pipeline for PBIN.
pub struct CompressionPipeline {
    /// Compression level.
    level: CompressionLevel,
    /// Whether to use BCJ filters.
    use_bcj: bool,
    /// Whether to use delta compression.
    use_delta: bool,
    /// Whether to train dictionaries.
    use_dict: bool,
    /// Trained dictionary (if any).
    dictionary: Option<TrainedDictionary>,
}

impl Default for CompressionPipeline {
    fn default() -> Self {
        Self::new(CompressionLevel::Balanced)
    }
}

impl CompressionPipeline {
    /// Create a new compression pipeline.
    pub fn new(level: CompressionLevel) -> Self {
        Self {
            level,
            use_bcj: true,
            use_delta: true,
            use_dict: true,
            dictionary: None,
        }
    }

    /// Disable BCJ filtering.
    pub fn without_bcj(mut self) -> Self {
        self.use_bcj = false;
        self
    }

    /// Disable delta compression.
    pub fn without_delta(mut self) -> Self {
        self.use_delta = false;
        self
    }

    /// Disable dictionary training.
    pub fn without_dict(mut self) -> Self {
        self.use_dict = false;
        self
    }

    /// Compress multiple binaries with the pipeline.
    pub fn compress_all(
        &mut self,
        binaries: Vec<(String, Vec<u8>)>,
    ) -> Result<CompressionResult> {
        if binaries.is_empty() {
            return Ok(CompressionResult {
                entries: Vec::new(),
                dictionary: None,
                stats: CompressionStats::default(),
            });
        }

        let mut stats = CompressionStats {
            original_size: binaries.iter().map(|(_, d)| d.len()).sum(),
            ..Default::default()
        };

        // Step 1: Parse binaries and apply BCJ filters
        let mut processed: Vec<(String, Vec<u8>)> = Vec::new();
        for (target, mut data) in binaries {
            if self.use_bcj {
                let arch = BcjArch::from_target(&target);
                if arch != BcjArch::None {
                    let mut filter = BcjFilter::new(arch);
                    filter.encode(&mut data)?;
                    stats.bcj_filtered += 1;
                }
            }
            processed.push((target, data));
        }

        // Step 2: Train dictionary if enabled
        if self.use_dict && processed.len() >= 4 {
            let samples: Vec<&[u8]> = processed.iter().map(|(_, d)| d.as_slice()).collect();
            match TrainedDictionary::train(&samples, DEFAULT_DICT_SIZE) {
                Ok(dict) => {
                    self.dictionary = Some(dict);
                    stats.dict_trained = true;
                }
                Err(_) => {
                    // Dictionary training failed, continue without
                }
            }
        }

        // Step 3: Group binaries for delta compression
        let groups = if self.use_delta {
            delta::group_by_similarity(&processed, self.level.delta_threshold())
        } else {
            // No grouping, each binary is its own group
            processed
                .iter()
                .map(|(target, _)| DeltaGroup {
                    reference_target: target.clone(),
                    delta_targets: Vec::new(),
                })
                .collect()
        };

        // Step 4: Compress each group
        let zstd_level = self.level.zstd_level();
        let mut entries: Vec<CompressedEntry> = Vec::new();

        // Build lookup for processed binaries
        let binary_map: HashMap<String, Vec<u8>> = processed.into_iter().collect();

        for group in groups {
            // Compress reference binary
            let ref_data = binary_map
                .get(&group.reference_target)
                .ok_or_else(|| CompressionError::InvalidData("Missing reference binary".into()))?;

            let compressed_ref = self.compress_single(ref_data, zstd_level)?;
            entries.push(CompressedEntry {
                target: group.reference_target.clone(),
                data: compressed_ref,
                bcj_filtered: self.use_bcj && BcjArch::from_target(&group.reference_target) != BcjArch::None,
                delta_reference: None,
                original_size: ref_data.len(),
            });

            // Compress delta targets
            for delta_target in &group.delta_targets {
                let target_data = binary_map
                    .get(delta_target)
                    .ok_or_else(|| CompressionError::InvalidData("Missing delta target".into()))?;

                // Create delta patch
                let patch = delta::create_patch(ref_data, target_data)?;

                // Compress the patch
                let compressed_patch = self.compress_single(&patch, zstd_level)?;

                // Only use delta if it's smaller than direct compression
                let direct_compressed = self.compress_single(target_data, zstd_level)?;

                if compressed_patch.len() < direct_compressed.len() {
                    stats.delta_used += 1;
                    entries.push(CompressedEntry {
                        target: delta_target.clone(),
                        data: compressed_patch,
                        bcj_filtered: self.use_bcj && BcjArch::from_target(delta_target) != BcjArch::None,
                        delta_reference: Some(group.reference_target.clone()),
                        original_size: target_data.len(),
                    });
                } else {
                    entries.push(CompressedEntry {
                        target: delta_target.clone(),
                        data: direct_compressed,
                        bcj_filtered: self.use_bcj && BcjArch::from_target(delta_target) != BcjArch::None,
                        delta_reference: None,
                        original_size: target_data.len(),
                    });
                }
            }
        }

        stats.compressed_size = entries.iter().map(|e| e.data.len()).sum();
        if let Some(ref dict) = self.dictionary {
            stats.compressed_size += dict.data.len();
        }

        Ok(CompressionResult {
            entries,
            dictionary: self.dictionary.as_ref().map(|d| d.data.clone()),
            stats,
        })
    }

    /// Compress a single binary.
    fn compress_single(&self, data: &[u8], level: i32) -> Result<Vec<u8>> {
        if let Some(ref dict) = self.dictionary {
            dict.compress(data, level)
        } else {
            dict::compress(data, level)
        }
    }
}

/// Result of compression pipeline.
#[derive(Debug)]
pub struct CompressionResult {
    /// Compressed entries.
    pub entries: Vec<CompressedEntry>,
    /// Trained dictionary (if any).
    pub dictionary: Option<Vec<u8>>,
    /// Compression statistics.
    pub stats: CompressionStats,
}

/// Compression statistics.
#[derive(Debug, Default)]
pub struct CompressionStats {
    /// Total original size.
    pub original_size: usize,
    /// Total compressed size (including dictionary).
    pub compressed_size: usize,
    /// Number of binaries with BCJ filter applied.
    pub bcj_filtered: usize,
    /// Number of binaries using delta compression.
    pub delta_used: usize,
    /// Whether dictionary was trained.
    pub dict_trained: bool,
}

impl CompressionStats {
    /// Calculate compression ratio.
    pub fn ratio(&self) -> f64 {
        if self.original_size == 0 {
            0.0
        } else {
            self.compressed_size as f64 / self.original_size as f64
        }
    }

    /// Calculate space savings percentage.
    pub fn savings_percent(&self) -> f64 {
        (1.0 - self.ratio()) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_binary(target: &str, seed: u8) -> (String, Vec<u8>) {
        let mut data = Vec::with_capacity(4096);

        // Simulate ELF header
        data.extend_from_slice(b"\x7FELF\x02\x01\x01\x00");
        data.extend_from_slice(&[0; 8]);

        // Add some x86-like instructions with CALL patterns
        for i in 0..500 {
            if i % 20 == 0 {
                // CALL instruction pattern
                data.push(0xE8);
                data.extend_from_slice(&[
                    (i as u8).wrapping_add(seed),
                    0x00,
                    0x00,
                    0x00,
                ]);
            } else {
                data.push((i as u8).wrapping_mul(seed.wrapping_add(1)));
            }
        }

        (target.to_string(), data)
    }

    #[test]
    fn test_compression_pipeline() {
        let binaries = vec![
            make_binary("linux-x86_64", 1),
            make_binary("darwin-x86_64", 2),
            make_binary("linux-aarch64", 3),
            make_binary("darwin-aarch64", 4),
        ];

        let original_size: usize = binaries.iter().map(|(_, d)| d.len()).sum();

        let mut pipeline = CompressionPipeline::new(CompressionLevel::Balanced);
        let result = pipeline.compress_all(binaries).unwrap();

        assert_eq!(result.entries.len(), 4);
        assert!(result.stats.compressed_size < original_size);

        println!("Original: {} bytes", result.stats.original_size);
        println!("Compressed: {} bytes", result.stats.compressed_size);
        println!("Ratio: {:.2}%", result.stats.ratio() * 100.0);
        println!("Savings: {:.2}%", result.stats.savings_percent());
    }

    #[test]
    fn test_tier_targets() {
        let core = PlatformTier::Core.targets();
        let standard = PlatformTier::Standard.targets();
        let extended = PlatformTier::Extended.targets();

        assert!(core.len() < standard.len());
        assert!(standard.len() < extended.len());

        // Core targets should be in standard and extended
        for target in &core {
            assert!(standard.contains(target));
            assert!(extended.contains(target));
        }
    }

    #[test]
    fn test_empty_input() {
        let mut pipeline = CompressionPipeline::new(CompressionLevel::Fast);
        let result = pipeline.compress_all(Vec::new()).unwrap();

        assert!(result.entries.is_empty());
        assert!(result.dictionary.is_none());
    }
}
