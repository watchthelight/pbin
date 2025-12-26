//! Delta compression using bsdiff algorithm.
//!
//! Uses the bidiff crate for creating delta patches between similar binaries.
//! This is particularly effective for binaries targeting the same architecture
//! but different operating systems (e.g., linux-x86_64 vs darwin-x86_64).

use crate::{CompressionError, Result};
use std::io::{Cursor, Read};

/// Create a delta patch between a reference binary and target binary.
///
/// The patch can be applied to the reference to recreate the target.
/// Useful for compressing similar binaries by storing only differences.
pub fn create_patch(reference: &[u8], target: &[u8]) -> Result<Vec<u8>> {
    let mut patch = Vec::new();
    bidiff::simple_diff(reference, target, &mut patch)
        .map_err(|e| CompressionError::Delta(format!("Failed to create patch: {}", e)))?;
    Ok(patch)
}

/// Apply a delta patch to a reference binary to recreate the target.
pub fn apply_patch(reference: &[u8], patch: &[u8]) -> Result<Vec<u8>> {
    let mut target = Vec::new();
    let patch_reader = Cursor::new(patch);
    let old_reader = Cursor::new(reference);

    let mut reader = bipatch::Reader::new(patch_reader, old_reader)
        .map_err(|e| CompressionError::Delta(format!("Failed to read patch: {}", e)))?;

    reader.read_to_end(&mut target)
        .map_err(|e| CompressionError::Delta(format!("Failed to apply patch: {}", e)))?;

    Ok(target)
}

/// Calculate the similarity ratio between two binaries.
///
/// Returns a value between 0.0 (completely different) and 1.0 (identical).
/// Used to decide whether delta compression is beneficial.
pub fn similarity_ratio(a: &[u8], b: &[u8]) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }

    // Quick length-based heuristic
    let len_ratio = a.len().min(b.len()) as f64 / a.len().max(b.len()) as f64;
    if len_ratio < 0.5 {
        return len_ratio * 0.5; // Very different sizes, low similarity
    }

    // Sample-based comparison for performance
    let sample_size = 1024.min(a.len().min(b.len()));
    let step = a.len().min(b.len()) / sample_size;

    let mut matches = 0;
    for i in 0..sample_size {
        let pos = i * step;
        if a.get(pos) == b.get(pos) {
            matches += 1;
        }
    }

    (matches as f64 / sample_size as f64) * len_ratio
}

/// Represents a group of similar binaries for delta compression.
#[derive(Debug)]
pub struct DeltaGroup {
    /// The reference binary (stored in full).
    pub reference_target: String,
    /// Targets that are stored as deltas from the reference.
    pub delta_targets: Vec<String>,
}

/// Group targets by similarity for delta compression.
///
/// Returns groups where the first target in each group is the reference
/// and remaining targets can be stored as deltas.
pub fn group_by_similarity(
    binaries: &[(String, Vec<u8>)],
    threshold: f64,
) -> Vec<DeltaGroup> {
    if binaries.is_empty() {
        return Vec::new();
    }

    let mut groups: Vec<DeltaGroup> = Vec::new();
    let mut assigned: Vec<bool> = vec![false; binaries.len()];

    // Group by architecture first (binaries of same arch are most similar)
    for (i, (target_i, data_i)) in binaries.iter().enumerate() {
        if assigned[i] {
            continue;
        }

        let arch_i = extract_arch(target_i);
        let mut group = DeltaGroup {
            reference_target: target_i.clone(),
            delta_targets: Vec::new(),
        };
        assigned[i] = true;

        // Find similar binaries
        for (j, (target_j, data_j)) in binaries.iter().enumerate() {
            if assigned[j] {
                continue;
            }

            // Same architecture is a strong indicator of similarity
            let arch_j = extract_arch(target_j);
            if arch_i == arch_j {
                let sim = similarity_ratio(data_i, data_j);
                if sim >= threshold {
                    group.delta_targets.push(target_j.clone());
                    assigned[j] = true;
                }
            }
        }

        groups.push(group);
    }

    groups
}

/// Extract architecture from target string (e.g., "linux-x86_64" -> "x86_64").
fn extract_arch(target: &str) -> &str {
    target.rsplit('-').next().unwrap_or(target)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_patch_roundtrip() {
        let reference = b"Hello, World! This is a test binary with some content.";
        let target = b"Hello, World! This is a modified binary with different content.";

        let patch = create_patch(reference, target).unwrap();
        let recovered = apply_patch(reference, &patch).unwrap();

        assert_eq!(recovered, target);
    }

    #[test]
    fn test_identical_patch() {
        // Verify that patching identical data works correctly
        let data: Vec<u8> = (0..1000).map(|i| (i % 256) as u8).collect();
        let patch = create_patch(&data, &data).unwrap();
        let recovered = apply_patch(&data, &patch).unwrap();

        assert_eq!(recovered, data);
        // Note: bidiff has overhead, so patches for identical data may not be smaller
        // The key property is that roundtrip works correctly
    }

    #[test]
    fn test_similarity_identical() {
        let data = vec![1, 2, 3, 4, 5];
        assert!((similarity_ratio(&data, &data) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_similarity_different() {
        let a = vec![0u8; 1000];
        let b = vec![255u8; 1000];
        let sim = similarity_ratio(&a, &b);
        assert!(sim < 0.1);
    }

    #[test]
    fn test_grouping() {
        let binaries = vec![
            ("linux-x86_64".to_string(), vec![1, 2, 3, 4]),
            ("darwin-x86_64".to_string(), vec![1, 2, 3, 5]),
            ("linux-aarch64".to_string(), vec![10, 20, 30, 40]),
            ("darwin-aarch64".to_string(), vec![10, 20, 30, 50]),
        ];

        let groups = group_by_similarity(&binaries, 0.5);

        // Should group x86_64 together and aarch64 together
        assert_eq!(groups.len(), 2);
    }
}
