//! PBIN Pack CLI
//!
//! Packs multiple platform-specific binaries into a single PBIN file.

use pbin_core::{blake3, Compression, PbinEntry, PbinHeader, PbinManifest, Target};
use pbin_stub::StubGenerator;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process;

const USAGE: &str = r#"pbin-pack - Pack binaries into PBIN format

USAGE:
    pbin-pack [OPTIONS]

OPTIONS:
    --name <NAME>               Application name (required)
    --version <VERSION>         Application version (default: 1.0.0)
    --output <PATH>             Output .pbin file (required)
    --linux-x86_64 <PATH>       Linux x86_64 binary
    --linux-aarch64 <PATH>      Linux aarch64 binary
    --linux-riscv64 <PATH>      Linux RISC-V 64 binary
    --darwin-x86_64 <PATH>      macOS x86_64 binary
    --darwin-aarch64 <PATH>     macOS aarch64 binary
    --windows-x86_64 <PATH>     Windows x86_64 binary (.exe)
    --windows-aarch64 <PATH>    Windows aarch64 binary (.exe)
    --help                      Show this help message

EXAMPLE:
    pbin-pack \
        --name hello \
        --version 1.0.0 \
        --linux-x86_64 ./target/x86_64-unknown-linux-gnu/release/hello \
        --darwin-aarch64 ./target/aarch64-apple-darwin/release/hello \
        --output hello.pbin
"#;

struct Config {
    name: String,
    version: String,
    output: PathBuf,
    binaries: HashMap<Target, PathBuf>,
}

fn parse_args() -> Result<Config, String> {
    let args: Vec<String> = std::env::args().collect();

    let mut name = None;
    let mut version = String::from("1.0.0");
    let mut output = None;
    let mut binaries = HashMap::new();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => {
                println!("{}", USAGE);
                process::exit(0);
            }
            "--name" => {
                i += 1;
                name = Some(args.get(i).ok_or("--name requires a value")?.clone());
            }
            "--version" => {
                i += 1;
                version = args.get(i).ok_or("--version requires a value")?.clone();
            }
            "--output" => {
                i += 1;
                output = Some(PathBuf::from(
                    args.get(i).ok_or("--output requires a value")?,
                ));
            }
            "--linux-x86_64" => {
                i += 1;
                binaries.insert(
                    Target::LinuxX86_64,
                    PathBuf::from(args.get(i).ok_or("--linux-x86_64 requires a value")?),
                );
            }
            "--linux-aarch64" => {
                i += 1;
                binaries.insert(
                    Target::LinuxAarch64,
                    PathBuf::from(args.get(i).ok_or("--linux-aarch64 requires a value")?),
                );
            }
            "--linux-riscv64" => {
                i += 1;
                binaries.insert(
                    Target::LinuxRiscv64,
                    PathBuf::from(args.get(i).ok_or("--linux-riscv64 requires a value")?),
                );
            }
            "--darwin-x86_64" => {
                i += 1;
                binaries.insert(
                    Target::DarwinX86_64,
                    PathBuf::from(args.get(i).ok_or("--darwin-x86_64 requires a value")?),
                );
            }
            "--darwin-aarch64" => {
                i += 1;
                binaries.insert(
                    Target::DarwinAarch64,
                    PathBuf::from(args.get(i).ok_or("--darwin-aarch64 requires a value")?),
                );
            }
            "--windows-x86_64" => {
                i += 1;
                binaries.insert(
                    Target::WindowsX86_64,
                    PathBuf::from(args.get(i).ok_or("--windows-x86_64 requires a value")?),
                );
            }
            "--windows-aarch64" => {
                i += 1;
                binaries.insert(
                    Target::WindowsAarch64,
                    PathBuf::from(args.get(i).ok_or("--windows-aarch64 requires a value")?),
                );
            }
            arg => {
                return Err(format!("Unknown argument: {}", arg));
            }
        }
        i += 1;
    }

    let name = name.ok_or("--name is required")?;
    let output = output.ok_or("--output is required")?;

    if binaries.is_empty() {
        return Err("At least one binary must be specified".to_string());
    }

    Ok(Config {
        name,
        version,
        output,
        binaries,
    })
}

fn read_binary(path: &PathBuf) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    Ok(data)
}

fn pack(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    println!("Packing {} v{}", config.name, config.version);

    // Read all binaries and compute checksums
    let mut binary_data: Vec<(Target, Vec<u8>, [u8; 32])> = Vec::new();

    for (target, path) in &config.binaries {
        println!("  Reading {} from {}", target, path.display());

        if !path.exists() {
            return Err(format!("Binary not found: {}", path.display()).into());
        }

        let data = read_binary(path)?;
        let checksum = blake3::hash(&data);

        println!(
            "    Size: {} bytes, Checksum: {}",
            data.len(),
            hex::encode(checksum.as_bytes())
        );

        binary_data.push((*target, data, *checksum.as_bytes()));
    }

    // Generate stub
    let stub = StubGenerator::generate();
    println!("  Stub size: {} bytes", stub.len());

    // Calculate offsets
    // Layout: stub (includes marker at end) + header (64) + manifest (variable) + binaries
    // The stub already ends with __PBIN_PAYLOAD__, so header starts at stub.len()
    let header_offset = stub.len();
    let manifest_offset = header_offset + 64;

    // Create manifest (we need to know binary offsets first)
    let mut manifest = PbinManifest::new(config.name, config.version);

    // First pass: calculate where each binary will be
    // We need to serialize the manifest to know its size, but the manifest contains offsets...
    // Solution: use placeholder offsets, serialize, then update

    for (target, data, checksum) in &binary_data {
        manifest.add_entry(PbinEntry::new(
            *target,
            0, // Placeholder
            data.len() as u64,
            data.len() as u64, // No compression for now
            *checksum,
        ));
    }

    // Serialize manifest to get its size
    let manifest_json = manifest.to_json()?;
    let manifest_size = manifest_json.len();

    // Now calculate actual offsets
    let mut current_offset = manifest_offset + manifest_size;
    for (i, (_, data, _)) in binary_data.iter().enumerate() {
        manifest.entries[i].offset = current_offset as u64;
        current_offset += data.len();
    }

    // Re-serialize with correct offsets
    let manifest_json = manifest.to_json()?;
    let manifest_bytes = manifest_json.as_bytes();

    // Verify manifest size didn't change (it shouldn't if offsets are similar length)
    // If it did, we need to recalculate
    if manifest_bytes.len() != manifest_size {
        // Recalculate with new size
        let new_manifest_size = manifest_bytes.len();
        let mut new_offset = manifest_offset + new_manifest_size;
        for (i, (_, data, _)) in binary_data.iter().enumerate() {
            manifest.entries[i].offset = new_offset as u64;
            new_offset += data.len();
        }
    }

    let manifest_json = manifest.to_json()?;
    let manifest_bytes = manifest_json.as_bytes();

    // Create header
    let header = PbinHeader::new(
        Compression::None, // No compression for now
        manifest.entries.len() as u8,
        manifest_bytes.len() as u32,
    );

    // Write output file
    let mut output = File::create(&config.output)?;

    // Write stub
    output.write_all(&stub)?;

    // Write header
    output.write_all(&header.to_bytes())?;

    // Write manifest
    output.write_all(manifest_bytes)?;

    // Write binaries
    for (target, data, _) in &binary_data {
        println!("  Writing {} ({} bytes)", target, data.len());
        output.write_all(data)?;
    }

    output.flush()?;

    // Make executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&config.output)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&config.output, perms)?;
    }

    let total_size = std::fs::metadata(&config.output)?.len();
    println!(
        "Created {} ({} bytes)",
        config.output.display(),
        total_size
    );

    Ok(())
}

// Simple hex encoder since we're not using external crate
mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

fn main() {
    let config = match parse_args() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: {}\n", e);
            eprintln!("{}", USAGE);
            process::exit(1);
        }
    };

    if let Err(e) = pack(config) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
