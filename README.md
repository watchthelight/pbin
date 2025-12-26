# PBIN - Polyglot Binary Format

[![CI](https://img.shields.io/github/actions/workflow/status/watchthelight/pbin/ci.yml?branch=main&label=CI)](https://github.com/watchthelight/pbin/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A revolutionary file format and toolchain that enables single-file, cross-platform executables. A `.pbin` file runs on any supported system with just `./file.pbin` (Unix) or double-click (Windows) - no installation, no runtime dependencies, no setup.

## Features

- **30 Platform Support**: One file runs on Linux, macOS, Windows, BSD, Android, iOS, and more
- **Smart Compression**: ~50-60% size reduction with zstd compression
- **Zero Dependencies**: Polyglot stub requires no external tools (except zstd for compressed mode)
- **Simple CLI**: Pack binaries with a single command

## Supported Platforms (30 Targets)

### Core Tier (CI Tested)
| Platform | Target | CI Status |
|----------|--------|-----------|
| ![Linux](https://img.shields.io/badge/Linux-x86__64-FCC624?logo=linux&logoColor=black) | `linux-x86_64` | ✅ Tested |
| ![Linux](https://img.shields.io/badge/Linux-aarch64-FCC624?logo=linux&logoColor=black) | `linux-aarch64` | ✅ Tested |
| ![macOS](https://img.shields.io/badge/macOS-x86__64-000000?logo=apple&logoColor=white) | `darwin-x86_64` | ✅ Rosetta |
| ![macOS](https://img.shields.io/badge/macOS-aarch64-000000?logo=apple&logoColor=white) | `darwin-aarch64` | ✅ Tested |
| ![Windows](https://img.shields.io/badge/Windows-x86__64-0078D6?logo=windows&logoColor=white) | `windows-x86_64` | ✅ Tested |
| ![Windows](https://img.shields.io/badge/Windows-aarch64-0078D6?logo=windows&logoColor=white) | `windows-aarch64` | 🔧 Supported |

### Standard Tier
| Platform | Target | Status |
|----------|--------|--------|
| ![Linux](https://img.shields.io/badge/Linux-x86__64__musl-FCC624?logo=linux&logoColor=black) | `linux-x86_64-musl` | 🔧 Supported |
| ![Linux](https://img.shields.io/badge/Linux-aarch64__musl-FCC624?logo=linux&logoColor=black) | `linux-aarch64-musl` | 🔧 Supported |
| ![Linux](https://img.shields.io/badge/Linux-armv7-FCC624?logo=linux&logoColor=black) | `linux-armv7` | 🔧 Supported |
| ![Linux](https://img.shields.io/badge/Linux-riscv64-FCC624?logo=linux&logoColor=black) | `linux-riscv64` | 🔧 Supported |
| ![Linux](https://img.shields.io/badge/Linux-ppc64le-FCC624?logo=linux&logoColor=black) | `linux-ppc64le` | 🔧 Supported |
| ![Linux](https://img.shields.io/badge/Linux-s390x-FCC624?logo=linux&logoColor=black) | `linux-s390x` | 🔧 Supported |
| ![Windows](https://img.shields.io/badge/Windows-x86-0078D6?logo=windows&logoColor=white) | `windows-x86` | 🔧 Supported |

### Extended Tier
| Platform | Target | Status |
|----------|--------|--------|
| ![FreeBSD](https://img.shields.io/badge/FreeBSD-x86__64-AB2B28?logo=freebsd&logoColor=white) | `freebsd-x86_64` | 🔧 Supported |
| ![FreeBSD](https://img.shields.io/badge/FreeBSD-aarch64-AB2B28?logo=freebsd&logoColor=white) | `freebsd-aarch64` | 🔧 Supported |
| ![NetBSD](https://img.shields.io/badge/NetBSD-x86__64-FF6600?logo=netbsd&logoColor=white) | `netbsd-x86_64` | 🔧 Supported |
| ![OpenBSD](https://img.shields.io/badge/OpenBSD-x86__64-F2CA30?logo=openbsd&logoColor=black) | `openbsd-x86_64` | 🔧 Supported |
| ![Android](https://img.shields.io/badge/Android-aarch64-3DDC84?logo=android&logoColor=white) | `android-aarch64` | 🔧 Supported |
| ![Android](https://img.shields.io/badge/Android-armv7-3DDC84?logo=android&logoColor=white) | `android-armv7` | 🔧 Supported |
| ![Android](https://img.shields.io/badge/Android-x86__64-3DDC84?logo=android&logoColor=white) | `android-x86_64` | 🔧 Supported |
| ![iOS](https://img.shields.io/badge/iOS-aarch64-000000?logo=ios&logoColor=white) | `ios-aarch64` | 🔧 Supported |
| ![Linux](https://img.shields.io/badge/Linux-mips64-FCC624?logo=linux&logoColor=black) | `linux-mips64` | 🔧 Supported |
| ![Linux](https://img.shields.io/badge/Linux-loongarch64-FCC624?logo=linux&logoColor=black) | `linux-loongarch64` | 🔧 Supported |
| ![WASI](https://img.shields.io/badge/WASI-wasm32-654FF0?logo=webassembly&logoColor=white) | `wasi-wasm32` | 🔧 Supported |

**Legend**: ✅ CI Tested | 🔧 Supported (not CI tested)

**Note**: Compressed PBINs require `zstd` on the target system. Uncompressed PBINs work everywhere with no dependencies.

## How It Works

PBIN uses a polyglot header strategy - a file that is simultaneously:
- A valid POSIX shell script
- A valid Windows batch file
- A container for compressed native executables

When executed, the polyglot stub:
1. Detects the current OS and architecture
2. Locates the correct embedded binary
3. Extracts and decompresses it to a temp location
4. Executes it with all original arguments
5. Cleans up on exit

## Quick Start

```bash
# Create a .pbin from pre-built binaries (with compression)
pbin-pack \
  --name myapp \
  --compress balanced \
  --linux-x86_64 ./target/x86_64-unknown-linux-gnu/release/myapp \
  --darwin-aarch64 ./target/aarch64-apple-darwin/release/myapp \
  --windows-x86_64 ./target/x86_64-pc-windows-msvc/release/myapp.exe \
  --output myapp.pbin

# Run it anywhere!
./myapp.pbin        # Linux/macOS
.\myapp.pbin        # Windows
```

## Compression Options

PBIN supports intelligent compression to significantly reduce file sizes:

```bash
# Balanced compression (default, ~50% reduction)
pbin-pack --compress balanced ...

# Maximum compression (slower, better ratio)
pbin-pack --compress maximum ...

# Fast compression (quick, larger files)
pbin-pack --compress fast ...

# No compression (fastest creation, largest files)
pbin-pack --no-compress ...
```

**Note**: Compressed PBINs require `zstd` to be installed on the target system. Uncompressed PBINs work everywhere with no dependencies.

## Building from Source

```bash
# Build all tools
cargo build --release

# Build test payload for available targets
./scripts/build-all-targets.sh

# Create example .pbin
./target/release/pbin-pack \
  --name hello \
  --compress balanced \
  --darwin-aarch64 ./target/pbin-payloads/hello-darwin-aarch64 \
  --output hello.pbin
```

## Project Structure

```
pbin/
├── spec/                    # Format specification
├── crates/
│   ├── pbin-core/          # Format parsing, manifest handling
│   ├── pbin-compress/      # Compression pipeline (zstd, BCJ, delta)
│   ├── pbin-pack/          # CLI: pack binaries into .pbin
│   ├── pbin-stub/          # Polyglot stub generator
│   └── pbin-unpack/        # CLI: extract/inspect .pbin files
├── stubs/                   # Stub templates
├── test-payload/           # Test programs
└── scripts/                # Build and CI scripts
```

## License

MIT License - see [LICENSE](LICENSE) for details.
