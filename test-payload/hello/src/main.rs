//! PBIN Test Payload - Hello
//!
//! A minimal cross-platform hello program that:
//! - Measures time from process start to output
//! - Detects OS and architecture
//! - Retrieves kernel/OS version
//! - Prompts user for confirmation
//!
//! Zero external dependencies - std only!

use std::env::consts::{ARCH, OS};
use std::io::{self, BufRead, Write};
use std::time::Instant;

fn main() {
    // Start timing immediately
    let start = Instant::now();

    // Get OS version info
    let version_info = get_version_info();

    // Calculate elapsed time
    let elapsed = start.elapsed();
    let nanos = elapsed.as_nanos();

    // Format the output
    let os_name = match OS {
        "linux" => "Linux",
        "macos" => "macOS",
        "windows" => "Windows",
        other => other,
    };

    let arch_name = match ARCH {
        "x86_64" => "x86_64",
        "aarch64" => "aarch64",
        "riscv64" => "riscv64",
        other => other,
    };

    // Print the detection message
    println!(
        "You're running me on {} {} ({}), I took {}ns to figure this out, hello!",
        os_name, arch_name, version_info, nanos
    );

    // Prompt for confirmation
    print!("Do I work? (yes/no): ");
    io::stdout().flush().expect("Failed to flush stdout");

    let stdin = io::stdin();
    let mut input = String::new();
    stdin.lock().read_line(&mut input).expect("Failed to read input");

    let response = input.trim().to_lowercase();
    if response == "yes" || response == "y" {
        std::process::exit(0);
    } else {
        std::process::exit(1);
    }
}

/// Gets OS/kernel version information.
fn get_version_info() -> String {
    #[cfg(target_os = "linux")]
    {
        get_linux_version()
    }

    #[cfg(target_os = "macos")]
    {
        get_macos_version()
    }

    #[cfg(target_os = "windows")]
    {
        get_windows_version()
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        "unknown".to_string()
    }
}

#[cfg(target_os = "linux")]
fn get_linux_version() -> String {
    // Try to read /proc/version
    if let Ok(content) = std::fs::read_to_string("/proc/version") {
        // Extract kernel version from "Linux version X.Y.Z ..."
        if let Some(version_part) = content.split_whitespace().nth(2) {
            return format!("kernel {}", version_part);
        }
    }

    // Fallback: try uname via reading /proc/sys/kernel/osrelease
    if let Ok(release) = std::fs::read_to_string("/proc/sys/kernel/osrelease") {
        return format!("kernel {}", release.trim());
    }

    "kernel unknown".to_string()
}

#[cfg(target_os = "macos")]
fn get_macos_version() -> String {
    // Read system version plist
    let plist_path = "/System/Library/CoreServices/SystemVersion.plist";
    if let Ok(content) = std::fs::read_to_string(plist_path) {
        // Simple XML parsing - look for ProductVersion
        if let Some(start) = content.find("<key>ProductVersion</key>") {
            let after_key = &content[start..];
            if let Some(string_start) = after_key.find("<string>") {
                let version_start = &after_key[string_start + 8..];
                if let Some(end) = version_start.find("</string>") {
                    return format!("macOS {}", &version_start[..end]);
                }
            }
        }
    }

    // Fallback: try reading kern.osrelease via sysctl
    "macOS unknown".to_string()
}

#[cfg(target_os = "windows")]
fn get_windows_version() -> String {
    // Use Windows API to get version info
    use std::mem::zeroed;

    #[repr(C)]
    #[allow(non_snake_case)]
    struct OSVERSIONINFOW {
        dwOSVersionInfoSize: u32,
        dwMajorVersion: u32,
        dwMinorVersion: u32,
        dwBuildNumber: u32,
        dwPlatformId: u32,
        szCSDVersion: [u16; 128],
    }

    #[link(name = "ntdll")]
    extern "system" {
        fn RtlGetVersion(lpVersionInformation: *mut OSVERSIONINFOW) -> i32;
    }

    unsafe {
        let mut info: OSVERSIONINFOW = zeroed();
        info.dwOSVersionInfoSize = std::mem::size_of::<OSVERSIONINFOW>() as u32;

        if RtlGetVersion(&mut info) == 0 {
            return format!(
                "Windows {}.{} (Build {})",
                info.dwMajorVersion, info.dwMinorVersion, info.dwBuildNumber
            );
        }
    }

    "Windows unknown".to_string()
}
