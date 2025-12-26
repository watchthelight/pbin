//! Target platform detection and representation.

/// Represents a supported target platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Target {
    LinuxX86_64,
    LinuxAarch64,
    LinuxRiscv64,
    DarwinX86_64,
    DarwinAarch64,
    WindowsX86_64,
    WindowsAarch64,
}

impl Target {
    /// Detects the current platform at runtime.
    pub fn detect_current() -> Option<Self> {
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        return Some(Target::LinuxX86_64);

        #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
        return Some(Target::LinuxAarch64);

        #[cfg(all(target_os = "linux", target_arch = "riscv64"))]
        return Some(Target::LinuxRiscv64);

        #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
        return Some(Target::DarwinX86_64);

        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        return Some(Target::DarwinAarch64);

        #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
        return Some(Target::WindowsX86_64);

        #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
        return Some(Target::WindowsAarch64);

        #[allow(unreachable_code)]
        None
    }

    /// Returns the string representation used in PBIN manifests.
    pub fn as_str(&self) -> &'static str {
        match self {
            Target::LinuxX86_64 => "linux-x86_64",
            Target::LinuxAarch64 => "linux-aarch64",
            Target::LinuxRiscv64 => "linux-riscv64",
            Target::DarwinX86_64 => "darwin-x86_64",
            Target::DarwinAarch64 => "darwin-aarch64",
            Target::WindowsX86_64 => "windows-x86_64",
            Target::WindowsAarch64 => "windows-aarch64",
        }
    }

    /// Parses a target string into a Target enum.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "linux-x86_64" => Some(Target::LinuxX86_64),
            "linux-aarch64" => Some(Target::LinuxAarch64),
            "linux-riscv64" => Some(Target::LinuxRiscv64),
            "darwin-x86_64" => Some(Target::DarwinX86_64),
            "darwin-aarch64" => Some(Target::DarwinAarch64),
            "windows-x86_64" => Some(Target::WindowsX86_64),
            "windows-aarch64" => Some(Target::WindowsAarch64),
            _ => None,
        }
    }

    /// Returns the Rust target triple for this target.
    pub fn rust_triple(&self) -> &'static str {
        match self {
            Target::LinuxX86_64 => "x86_64-unknown-linux-gnu",
            Target::LinuxAarch64 => "aarch64-unknown-linux-gnu",
            Target::LinuxRiscv64 => "riscv64gc-unknown-linux-gnu",
            Target::DarwinX86_64 => "x86_64-apple-darwin",
            Target::DarwinAarch64 => "aarch64-apple-darwin",
            Target::WindowsX86_64 => "x86_64-pc-windows-msvc",
            Target::WindowsAarch64 => "aarch64-pc-windows-msvc",
        }
    }

    /// Returns all supported targets.
    pub fn all() -> &'static [Target] {
        &[
            Target::LinuxX86_64,
            Target::LinuxAarch64,
            Target::LinuxRiscv64,
            Target::DarwinX86_64,
            Target::DarwinAarch64,
            Target::WindowsX86_64,
            Target::WindowsAarch64,
        ]
    }
}

impl std::fmt::Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
