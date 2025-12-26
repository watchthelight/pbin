//! Polyglot stub generator.

/// The embedded polyglot stub template.
/// This template works as both a POSIX shell script and a Windows batch file.
pub const STUB_TEMPLATE: &str = include_str!("../../../stubs/polyglot.template");

/// Generates polyglot stubs that work as both shell scripts and batch files.
pub struct StubGenerator;

impl StubGenerator {
    /// Returns the polyglot stub as bytes.
    ///
    /// The stub is a script that:
    /// 1. Detects the current OS and architecture
    /// 2. Finds the payload marker in the file
    /// 3. Reads the PBIN header and manifest
    /// 4. Extracts the appropriate binary for the current platform
    /// 5. Executes it with all original arguments
    /// 6. Cleans up temporary files
    pub fn generate() -> Vec<u8> {
        STUB_TEMPLATE.as_bytes().to_vec()
    }

    /// Returns the stub size in bytes.
    pub fn stub_size() -> usize {
        STUB_TEMPLATE.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stub_generation() {
        let stub = StubGenerator::generate();
        assert!(!stub.is_empty());

        // Verify it contains the payload marker at the end
        let stub_str = String::from_utf8_lossy(&stub);
        assert!(stub_str.ends_with("__PBIN_PAYLOAD__"));

        // Verify it starts with the batch/shell polyglot
        assert!(stub_str.starts_with(":<<"));
    }

    #[test]
    fn test_stub_size() {
        let size = StubGenerator::stub_size();
        // Stub should be under 4KB as per spec
        assert!(size < 4096, "Stub size {} exceeds 4KB limit", size);
    }
}
