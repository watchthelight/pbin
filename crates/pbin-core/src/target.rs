//! Target platform detection and representation.

/// Represents a supported target platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Target {
    // Linux variants
    LinuxX86_64,
    LinuxAarch64,
    LinuxRiscv64,
    LinuxArmv7,
    LinuxPpc64le,
    LinuxS390x,
    LinuxMips64,
    LinuxI686,
    LinuxLoongarch64,

    // macOS
    DarwinX86_64,
    DarwinAarch64,

    // Windows
    WindowsX86_64,
    WindowsAarch64,
    WindowsX86,

    // BSD
    FreebsdX86_64,
    FreebsdAarch64,
    NetbsdX86_64,
    OpenbsdX86_64,

    // Mobile
    AndroidAarch64,
    AndroidArmv7,
    AndroidX86_64,
    IosAarch64,

    // WebAssembly
    WasiWasm32,
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

        #[cfg(all(target_os = "linux", target_arch = "arm"))]
        return Some(Target::LinuxArmv7);

        #[cfg(all(target_os = "linux", target_arch = "powerpc64"))]
        return Some(Target::LinuxPpc64le);

        #[cfg(all(target_os = "linux", target_arch = "s390x"))]
        return Some(Target::LinuxS390x);

        #[cfg(all(target_os = "linux", target_arch = "mips64"))]
        return Some(Target::LinuxMips64);

        #[cfg(all(target_os = "linux", target_arch = "x86"))]
        return Some(Target::LinuxI686);

        #[cfg(all(target_os = "linux", target_arch = "loongarch64"))]
        return Some(Target::LinuxLoongarch64);

        #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
        return Some(Target::DarwinX86_64);

        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        return Some(Target::DarwinAarch64);

        #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
        return Some(Target::WindowsX86_64);

        #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
        return Some(Target::WindowsAarch64);

        #[cfg(all(target_os = "windows", target_arch = "x86"))]
        return Some(Target::WindowsX86);

        #[cfg(all(target_os = "freebsd", target_arch = "x86_64"))]
        return Some(Target::FreebsdX86_64);

        #[cfg(all(target_os = "freebsd", target_arch = "aarch64"))]
        return Some(Target::FreebsdAarch64);

        #[cfg(all(target_os = "netbsd", target_arch = "x86_64"))]
        return Some(Target::NetbsdX86_64);

        #[cfg(all(target_os = "openbsd", target_arch = "x86_64"))]
        return Some(Target::OpenbsdX86_64);

        #[cfg(all(target_os = "android", target_arch = "aarch64"))]
        return Some(Target::AndroidAarch64);

        #[cfg(all(target_os = "android", target_arch = "arm"))]
        return Some(Target::AndroidArmv7);

        #[cfg(all(target_os = "android", target_arch = "x86_64"))]
        return Some(Target::AndroidX86_64);

        #[cfg(all(target_os = "ios", target_arch = "aarch64"))]
        return Some(Target::IosAarch64);

        #[cfg(target_os = "wasi")]
        return Some(Target::WasiWasm32);

        #[allow(unreachable_code)]
        None
    }

    /// Returns the string representation used in PBIN manifests.
    pub fn as_str(&self) -> &'static str {
        match self {
            Target::LinuxX86_64 => "linux-x86_64",
            Target::LinuxAarch64 => "linux-aarch64",
            Target::LinuxRiscv64 => "linux-riscv64",
            Target::LinuxArmv7 => "linux-armv7",
            Target::LinuxPpc64le => "linux-ppc64le",
            Target::LinuxS390x => "linux-s390x",
            Target::LinuxMips64 => "linux-mips64",
            Target::LinuxI686 => "linux-i686",
            Target::LinuxLoongarch64 => "linux-loongarch64",
            Target::DarwinX86_64 => "darwin-x86_64",
            Target::DarwinAarch64 => "darwin-aarch64",
            Target::WindowsX86_64 => "windows-x86_64",
            Target::WindowsAarch64 => "windows-aarch64",
            Target::WindowsX86 => "windows-x86",
            Target::FreebsdX86_64 => "freebsd-x86_64",
            Target::FreebsdAarch64 => "freebsd-aarch64",
            Target::NetbsdX86_64 => "netbsd-x86_64",
            Target::OpenbsdX86_64 => "openbsd-x86_64",
            Target::AndroidAarch64 => "android-aarch64",
            Target::AndroidArmv7 => "android-armv7",
            Target::AndroidX86_64 => "android-x86_64",
            Target::IosAarch64 => "ios-aarch64",
            Target::WasiWasm32 => "wasi-wasm32",
        }
    }

    /// Parses a target string into a Target enum.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "linux-x86_64" => Some(Target::LinuxX86_64),
            "linux-aarch64" => Some(Target::LinuxAarch64),
            "linux-riscv64" => Some(Target::LinuxRiscv64),
            "linux-armv7" => Some(Target::LinuxArmv7),
            "linux-ppc64le" => Some(Target::LinuxPpc64le),
            "linux-s390x" => Some(Target::LinuxS390x),
            "linux-mips64" => Some(Target::LinuxMips64),
            "linux-i686" => Some(Target::LinuxI686),
            "linux-loongarch64" => Some(Target::LinuxLoongarch64),
            "darwin-x86_64" => Some(Target::DarwinX86_64),
            "darwin-aarch64" => Some(Target::DarwinAarch64),
            "windows-x86_64" => Some(Target::WindowsX86_64),
            "windows-aarch64" => Some(Target::WindowsAarch64),
            "windows-x86" => Some(Target::WindowsX86),
            "freebsd-x86_64" => Some(Target::FreebsdX86_64),
            "freebsd-aarch64" => Some(Target::FreebsdAarch64),
            "netbsd-x86_64" => Some(Target::NetbsdX86_64),
            "openbsd-x86_64" => Some(Target::OpenbsdX86_64),
            "android-aarch64" => Some(Target::AndroidAarch64),
            "android-armv7" => Some(Target::AndroidArmv7),
            "android-x86_64" => Some(Target::AndroidX86_64),
            "ios-aarch64" => Some(Target::IosAarch64),
            "wasi-wasm32" => Some(Target::WasiWasm32),
            _ => None,
        }
    }

    /// Returns the Rust target triple for this target.
    pub fn rust_triple(&self) -> &'static str {
        match self {
            Target::LinuxX86_64 => "x86_64-unknown-linux-gnu",
            Target::LinuxAarch64 => "aarch64-unknown-linux-gnu",
            Target::LinuxRiscv64 => "riscv64gc-unknown-linux-gnu",
            Target::LinuxArmv7 => "armv7-unknown-linux-gnueabihf",
            Target::LinuxPpc64le => "powerpc64le-unknown-linux-gnu",
            Target::LinuxS390x => "s390x-unknown-linux-gnu",
            Target::LinuxMips64 => "mips64-unknown-linux-gnuabi64",
            Target::LinuxI686 => "i686-unknown-linux-gnu",
            Target::LinuxLoongarch64 => "loongarch64-unknown-linux-gnu",
            Target::DarwinX86_64 => "x86_64-apple-darwin",
            Target::DarwinAarch64 => "aarch64-apple-darwin",
            Target::WindowsX86_64 => "x86_64-pc-windows-msvc",
            Target::WindowsAarch64 => "aarch64-pc-windows-msvc",
            Target::WindowsX86 => "i686-pc-windows-msvc",
            Target::FreebsdX86_64 => "x86_64-unknown-freebsd",
            Target::FreebsdAarch64 => "aarch64-unknown-freebsd",
            Target::NetbsdX86_64 => "x86_64-unknown-netbsd",
            Target::OpenbsdX86_64 => "x86_64-unknown-openbsd",
            Target::AndroidAarch64 => "aarch64-linux-android",
            Target::AndroidArmv7 => "armv7-linux-androideabi",
            Target::AndroidX86_64 => "x86_64-linux-android",
            Target::IosAarch64 => "aarch64-apple-ios",
            Target::WasiWasm32 => "wasm32-wasip1",
        }
    }

    /// Returns all supported targets.
    pub fn all() -> &'static [Target] {
        &[
            Target::LinuxX86_64,
            Target::LinuxAarch64,
            Target::LinuxRiscv64,
            Target::LinuxArmv7,
            Target::LinuxPpc64le,
            Target::LinuxS390x,
            Target::LinuxMips64,
            Target::LinuxI686,
            Target::LinuxLoongarch64,
            Target::DarwinX86_64,
            Target::DarwinAarch64,
            Target::WindowsX86_64,
            Target::WindowsAarch64,
            Target::WindowsX86,
            Target::FreebsdX86_64,
            Target::FreebsdAarch64,
            Target::NetbsdX86_64,
            Target::OpenbsdX86_64,
            Target::AndroidAarch64,
            Target::AndroidArmv7,
            Target::AndroidX86_64,
            Target::IosAarch64,
            Target::WasiWasm32,
        ]
    }
}

impl std::fmt::Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
