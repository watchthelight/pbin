//! PBIN Pack CLI
//!
//! Packs multiple platform-specific binaries into a single PBIN file.

use pbin_compress::{CompressionLevel, CompressionPipeline};
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

    Platform binaries:
    --linux-x86_64 <PATH>       Linux x86_64 binary
    --linux-aarch64 <PATH>      Linux aarch64 binary
    --linux-riscv64 <PATH>      Linux RISC-V 64 binary
    --darwin-x86_64 <PATH>      macOS x86_64 binary
    --darwin-aarch64 <PATH>     macOS aarch64 binary
    --windows-x86_64 <PATH>     Windows x86_64 binary (.exe)
    --windows-aarch64 <PATH>    Windows aarch64 binary (.exe)

    Compression options:
    --compress <LEVEL>          Compression level: fast, balanced, maximum (default: balanced)
    --no-compress               Disable compression entirely
    --no-bcj                    Disable BCJ preprocessing filter
    --no-delta                  Disable delta compression
    --no-dict                   Disable dictionary training

    --help                      Show this help message

EXAMPLE:
    pbin-pack \
        --name hello \
        --version 1.0.0 \
        --compress balanced \
        --linux-x86_64 ./target/x86_64-unknown-linux-gnu/release/hello \
        --darwin-aarch64 ./target/aarch64-apple-darwin/release/hello \
        --output hello.pbin
"#;

struct Config {
    name: String,
    version: String,
    output: PathBuf,
    binaries: HashMap<Target, PathBuf>,
    compression_level: Option<CompressionLevel>,
    use_bcj: bool,
    use_delta: bool,
    use_dict: bool,
}

fn parse_args() -> Result<Config, String> {
    let args: Vec<String> = std::env::args().collect();

    let mut name = None;
    let mut version = String::from("1.0.0");
    let mut output = None;
    let mut binaries = HashMap::new();
    let mut compression_level = Some(CompressionLevel::Balanced);
    let mut use_bcj = true;
    let mut use_delta = true;
    let mut use_dict = true;

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
            "--compress" => {
                i += 1;
                let level_str = args.get(i).ok_or("--compress requires a value")?;
                compression_level = Some(match level_str.as_str() {
                    "fast" => CompressionLevel::Fast,
                    "balanced" => CompressionLevel::Balanced,
                    "maximum" | "max" => CompressionLevel::Maximum,
                    _ => return Err(format!("Unknown compression level: {}", level_str)),
                });
            }
            "--no-compress" => {
                compression_level = None;
            }
            "--no-bcj" => {
                use_bcj = false;
            }
            "--no-delta" => {
                use_delta = false;
            }
            "--no-dict" => {
                use_dict = false;
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
        compression_level,
        use_bcj,
        use_delta,
        use_dict,
    })
}

fn read_binary(path: &PathBuf) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    Ok(data)
}

fn target_to_string(target: Target) -> String {
    match target {
        Target::LinuxX86_64 => "linux-x86_64".to_string(),
        Target::LinuxAarch64 => "linux-aarch64".to_string(),
        Target::LinuxRiscv64 => "linux-riscv64".to_string(),
        Target::DarwinX86_64 => "darwin-x86_64".to_string(),
        Target::DarwinAarch64 => "darwin-aarch64".to_string(),
        Target::WindowsX86_64 => "windows-x86_64".to_string(),
        Target::WindowsAarch64 => "windows-aarch64".to_string(),
    }
}

fn pack(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    println!("Packing {} v{}", config.name, config.version);

    // Read all binaries
    let mut binary_data: Vec<(Target, Vec<u8>)> = Vec::new();
    let mut total_original_size = 0usize;

    for (target, path) in &config.binaries {
        println!("  Reading {} from {}", target, path.display());

        if !path.exists() {
            return Err(format!("Binary not found: {}", path.display()).into());
        }

        let data = read_binary(path)?;
        total_original_size += data.len();
        println!("    Size: {} bytes", data.len());

        binary_data.push((*target, data));
    }

    // Prepare for compression
    let compression_type: Compression;
    let compressed_entries: Vec<(Target, Vec<u8>, [u8; 32])>;

    if let Some(level) = config.compression_level {
        println!(
            "\n  Compressing with {:?} level (bcj={}, delta={}, dict={})...",
            level, config.use_bcj, config.use_delta, config.use_dict
        );

        // Prepare binaries for compression pipeline
        let binaries_for_compression: Vec<(String, Vec<u8>)> = binary_data
            .iter()
            .map(|(target, data)| (target_to_string(*target), data.clone()))
            .collect();

        // Create and configure pipeline
        let mut pipeline = CompressionPipeline::new(level);
        if !config.use_bcj {
            pipeline = pipeline.without_bcj();
        }
        if !config.use_delta {
            pipeline = pipeline.without_delta();
        }
        if !config.use_dict {
            pipeline = pipeline.without_dict();
        }

        // Compress all binaries
        let result = pipeline.compress_all(binaries_for_compression)?;

        println!("    Original: {} bytes", result.stats.original_size);
        println!("    Compressed: {} bytes", result.stats.compressed_size);
        println!(
            "    Ratio: {:.1}% (saved {:.1}%)",
            result.stats.ratio() * 100.0,
            result.stats.savings_percent()
        );
        if result.stats.bcj_filtered > 0 {
            println!("    BCJ filtered: {} binaries", result.stats.bcj_filtered);
        }
        if result.stats.delta_used > 0 {
            println!("    Delta compressed: {} binaries", result.stats.delta_used);
        }
        if result.stats.dict_trained {
            println!(
                "    Dictionary: {} bytes",
                result.dictionary.as_ref().map(|d| d.len()).unwrap_or(0)
            );
        }

        compression_type = Compression::Zstd;

        // Map compressed entries back to Target
        compressed_entries = binary_data
            .iter()
            .map(|(target, _original_data)| {
                let target_str = target_to_string(*target);
                let entry = result
                    .entries
                    .iter()
                    .find(|e| e.target == target_str)
                    .expect("Missing compressed entry");
                let checksum = blake3::hash(&entry.data);
                (*target, entry.data.clone(), *checksum.as_bytes())
            })
            .collect();
    } else {
        println!("\n  Compression disabled");
        compression_type = Compression::None;

        compressed_entries = binary_data
            .into_iter()
            .map(|(target, data)| {
                let checksum = blake3::hash(&data);
                (target, data, *checksum.as_bytes())
            })
            .collect();
    }

    // Generate stub
    let stub = StubGenerator::generate();
    println!("\n  Stub size: {} bytes", stub.len());

    // Calculate offsets
    let header_offset = stub.len();
    let manifest_offset = header_offset + 64;

    // Create manifest with placeholder offsets
    let mut manifest = PbinManifest::new(config.name, config.version);

    for (target, data, checksum) in &compressed_entries {
        manifest.add_entry(PbinEntry::new(
            *target,
            0, // Placeholder
            data.len() as u64,
            data.len() as u64,
            *checksum,
        ));
    }

    // Calculate actual offsets
    let manifest_json = manifest.to_json()?;
    let manifest_size = manifest_json.len();

    let mut current_offset = manifest_offset + manifest_size;
    for (i, (_, data, _)) in compressed_entries.iter().enumerate() {
        manifest.entries[i].offset = current_offset as u64;
        current_offset += data.len();
    }

    // Re-serialize with correct offsets
    let manifest_json = manifest.to_json()?;
    let manifest_bytes = manifest_json.as_bytes();

    // Handle size change
    if manifest_bytes.len() != manifest_size {
        let new_manifest_size = manifest_bytes.len();
        let mut new_offset = manifest_offset + new_manifest_size;
        for (i, (_, data, _)) in compressed_entries.iter().enumerate() {
            manifest.entries[i].offset = new_offset as u64;
            new_offset += data.len();
        }
    }

    let manifest_json = manifest.to_json()?;
    let manifest_bytes = manifest_json.as_bytes();

    // Create header
    let header = PbinHeader::new(
        compression_type,
        manifest.entries.len() as u8,
        manifest_bytes.len() as u32,
    );

    // Write output file
    let mut output = File::create(&config.output)?;

    output.write_all(&stub)?;
    output.write_all(&header.to_bytes())?;
    output.write_all(manifest_bytes)?;

    for (target, data, _) in &compressed_entries {
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
        "\nCreated {} ({} bytes, {:.1}% of original)",
        config.output.display(),
        total_size,
        (total_size as f64 / total_original_size as f64) * 100.0
    );

    Ok(())
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
