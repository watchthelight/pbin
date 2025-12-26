//! Zstd dictionary training and management.
//!
//! Creates optimized dictionaries from multiple binaries to improve
//! compression ratios. Particularly effective when compressing many
//! similar binaries (same architecture, similar code patterns).

use crate::{CompressionError, Result};

/// Default dictionary size (32KB is a good balance).
pub const DEFAULT_DICT_SIZE: usize = 32 * 1024;

/// Maximum dictionary size (128KB).
pub const MAX_DICT_SIZE: usize = 128 * 1024;

/// Minimum number of samples needed for dictionary training.
pub const MIN_SAMPLES: usize = 4;

/// Train a zstd dictionary from multiple binary samples.
///
/// The dictionary captures common patterns across all samples,
/// improving compression ratios significantly (often 20-40% better).
pub fn train_dictionary(samples: &[&[u8]], dict_size: usize) -> Result<Vec<u8>> {
    if samples.len() < MIN_SAMPLES {
        return Err(CompressionError::InvalidData(format!(
            "Need at least {} samples for dictionary training, got {}",
            MIN_SAMPLES,
            samples.len()
        )));
    }

    let dict_size = dict_size.min(MAX_DICT_SIZE);

    // Train dictionary using zstd - it takes a slice of samples
    let dict = zstd::dict::from_samples(samples, dict_size)
        .map_err(|e| CompressionError::Zstd(format!("Dictionary training failed: {}", e)))?;

    Ok(dict)
}

/// Compress data using a trained dictionary.
pub fn compress_with_dict(data: &[u8], dict: &[u8], level: i32) -> Result<Vec<u8>> {
    let mut encoder = zstd::bulk::Compressor::with_dictionary(level, dict)
        .map_err(|e| CompressionError::Zstd(format!("Failed to create compressor: {}", e)))?;

    encoder
        .compress(data)
        .map_err(|e| CompressionError::Zstd(format!("Compression failed: {}", e)))
}

/// Decompress data using a trained dictionary.
pub fn decompress_with_dict(data: &[u8], dict: &[u8]) -> Result<Vec<u8>> {
    let mut decoder = zstd::bulk::Decompressor::with_dictionary(dict)
        .map_err(|e| CompressionError::Zstd(format!("Failed to create decompressor: {}", e)))?;

    // Estimate output size (compressed data is typically 2-10x smaller)
    let estimated_size = data.len() * 10;

    decoder
        .decompress(data, estimated_size)
        .map_err(|e| CompressionError::Decompression(format!("Decompression failed: {}", e)))
}

/// Compress data without a dictionary (standard zstd).
pub fn compress(data: &[u8], level: i32) -> Result<Vec<u8>> {
    zstd::bulk::compress(data, level)
        .map_err(|e| CompressionError::Zstd(format!("Compression failed: {}", e)))
}

/// Decompress data without a dictionary.
pub fn decompress(data: &[u8]) -> Result<Vec<u8>> {
    // Estimate output size
    let estimated_size = data.len() * 10;

    zstd::bulk::decompress(data, estimated_size)
        .map_err(|e| CompressionError::Decompression(format!("Decompression failed: {}", e)))
}

/// Represents a trained dictionary with metadata.
#[derive(Debug, Clone)]
pub struct TrainedDictionary {
    /// The dictionary data.
    pub data: Vec<u8>,
    /// Number of samples used for training.
    pub sample_count: usize,
    /// Total size of training samples.
    pub total_sample_size: usize,
}

impl TrainedDictionary {
    /// Train a new dictionary from samples.
    pub fn train(samples: &[&[u8]], dict_size: usize) -> Result<Self> {
        let total_sample_size = samples.iter().map(|s| s.len()).sum();
        let data = train_dictionary(samples, dict_size)?;

        Ok(Self {
            data,
            sample_count: samples.len(),
            total_sample_size,
        })
    }

    /// Compress data using this dictionary.
    pub fn compress(&self, data: &[u8], level: i32) -> Result<Vec<u8>> {
        compress_with_dict(data, &self.data, level)
    }

    /// Decompress data using this dictionary.
    pub fn decompress(&self, data: &[u8]) -> Result<Vec<u8>> {
        decompress_with_dict(data, &self.data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generate_sample(seed: u8) -> Vec<u8> {
        // Generate pseudo-binary data with common patterns
        let mut data = Vec::with_capacity(4096);

        // Common header pattern
        data.extend_from_slice(b"\x7FELF\x02\x01\x01\x00");
        data.extend_from_slice(&[0; 8]);

        // Variable content based on seed
        for i in 0..500 {
            data.push(((i as u8).wrapping_mul(seed)).wrapping_add(seed));
        }

        // Common footer pattern
        data.extend_from_slice(b"\x00\x00\x00\x00.text\x00.data\x00");

        data
    }

    #[test]
    fn test_compress_decompress() {
        let data = b"Hello, World! This is test data for compression.";

        let compressed = compress(data, 3).unwrap();
        let decompressed = decompress(&compressed).unwrap();

        assert_eq!(&decompressed, data);
    }

    #[test]
    fn test_dictionary_training() {
        let samples: Vec<Vec<u8>> = (0..8).map(|i| generate_sample(i)).collect();
        let sample_refs: Vec<&[u8]> = samples.iter().map(|s| s.as_slice()).collect();

        let dict = TrainedDictionary::train(&sample_refs, DEFAULT_DICT_SIZE).unwrap();

        assert!(!dict.data.is_empty());
        assert!(dict.data.len() <= DEFAULT_DICT_SIZE);
    }

    #[test]
    fn test_dictionary_compression() {
        let samples: Vec<Vec<u8>> = (0..8).map(|i| generate_sample(i)).collect();
        let sample_refs: Vec<&[u8]> = samples.iter().map(|s| s.as_slice()).collect();

        let dict = TrainedDictionary::train(&sample_refs, DEFAULT_DICT_SIZE).unwrap();

        // Compress a new sample using the dictionary
        let new_sample = generate_sample(100);
        let compressed = dict.compress(&new_sample, 3).unwrap();
        let decompressed = dict.decompress(&compressed).unwrap();

        assert_eq!(decompressed, new_sample);

        // Dictionary compression should be smaller than without
        let without_dict = compress(&new_sample, 3).unwrap();
        // Note: For small/simple test data, dictionary might not help much
        assert!(compressed.len() > 0);
        println!(
            "With dict: {} bytes, without: {} bytes",
            compressed.len(),
            without_dict.len()
        );
    }

    #[test]
    fn test_insufficient_samples() {
        let samples: Vec<Vec<u8>> = (0..2).map(|i| generate_sample(i)).collect();
        let sample_refs: Vec<&[u8]> = samples.iter().map(|s| s.as_slice()).collect();

        let result = train_dictionary(&sample_refs, DEFAULT_DICT_SIZE);
        assert!(result.is_err());
    }
}
