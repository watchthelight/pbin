# PBIN - Polyglot Binary Format

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust Tests](https://img.shields.io/github/actions/workflow/status/watchthelight/pbin/ci.yml?branch=main&label=Tests)](https://github.com/watchthelight/pbin/actions/workflows/ci.yml)

A revolutionary file format and toolchain that enables single-file, cross-platform executables. A `.pbin` file runs on any supported system with just `./file.pbin` (Unix) or double-click (Windows) - no installation, no runtime dependencies, no setup.

## Features

- **26+ Platform Support**: One file runs on Linux, macOS, Windows, BSD, Android, iOS, and more
- **Smart Compression**: ~50-60% size reduction with zstd compression
- **Zero Dependencies**: Polyglot stub requires no external tools (except zstd for compressed mode)
- **Simple CLI**: Pack binaries with a single command

## Platform CI Status

### Core Tier
| | | | | | |
|:-:|:-:|:-:|:-:|:-:|:-:|
| [![Linux x86_64](https://img.shields.io/github/actions/workflow/status/watchthelight/pbin/ci.yml?branch=main&label=Linux%20x86_64)](https://github.com/watchthelight/pbin/actions/workflows/ci.yml) | [![Linux aarch64](https://img.shields.io/github/actions/workflow/status/watchthelight/pbin/ci.yml?branch=main&label=Linux%20aarch64)](https://github.com/watchthelight/pbin/actions/workflows/ci.yml) | [![macOS x86_64](https://img.shields.io/github/actions/workflow/status/watchthelight/pbin/ci.yml?branch=main&label=macOS%20x86_64)](https://github.com/watchthelight/pbin/actions/workflows/ci.yml) | [![macOS aarch64](https://img.shields.io/github/actions/workflow/status/watchthelight/pbin/ci.yml?branch=main&label=macOS%20aarch64)](https://github.com/watchthelight/pbin/actions/workflows/ci.yml) | [![Windows x86_64](https://img.shields.io/github/actions/workflow/status/watchthelight/pbin/ci.yml?branch=main&label=Windows%20x86_64)](https://github.com/watchthelight/pbin/actions/workflows/ci.yml) | [![Windows aarch64](https://img.shields.io/github/actions/workflow/status/watchthelight/pbin/ci.yml?branch=main&label=Windows%20aarch64)](https://github.com/watchthelight/pbin/actions/workflows/ci.yml) |

### Standard Tier
| | | | | | | |
|:-:|:-:|:-:|:-:|:-:|:-:|:-:|
| [![Linux x86_64 musl](https://img.shields.io/github/actions/workflow/status/watchthelight/pbin/ci.yml?branch=main&label=Linux%20musl%20x86_64)](https://github.com/watchthelight/pbin/actions/workflows/ci.yml) | [![Linux aarch64 musl](https://img.shields.io/github/actions/workflow/status/watchthelight/pbin/ci.yml?branch=main&label=Linux%20musl%20aarch64)](https://github.com/watchthelight/pbin/actions/workflows/ci.yml) | [![Linux armv7](https://img.shields.io/github/actions/workflow/status/watchthelight/pbin/ci.yml?branch=main&label=Linux%20armv7)](https://github.com/watchthelight/pbin/actions/workflows/ci.yml) | [![Linux riscv64](https://img.shields.io/github/actions/workflow/status/watchthelight/pbin/ci.yml?branch=main&label=Linux%20riscv64)](https://github.com/watchthelight/pbin/actions/workflows/ci.yml) | [![Linux ppc64le](https://img.shields.io/github/actions/workflow/status/watchthelight/pbin/ci.yml?branch=main&label=Linux%20ppc64le)](https://github.com/watchthelight/pbin/actions/workflows/ci.yml) | [![Linux s390x](https://img.shields.io/github/actions/workflow/status/watchthelight/pbin/ci.yml?branch=main&label=Linux%20s390x)](https://github.com/watchthelight/pbin/actions/workflows/ci.yml) | [![Windows x86](https://img.shields.io/github/actions/workflow/status/watchthelight/pbin/ci.yml?branch=main&label=Windows%20x86)](https://github.com/watchthelight/pbin/actions/workflows/ci.yml) |

### Extended Tier
| | | | | |
|:-:|:-:|:-:|:-:|:-:|
| [![FreeBSD x86_64](https://img.shields.io/github/actions/workflow/status/watchthelight/pbin/ci.yml?branch=main&label=FreeBSD%20x86_64)](https://github.com/watchthelight/pbin/actions/workflows/ci.yml) | [![FreeBSD aarch64](https://img.shields.io/github/actions/workflow/status/watchthelight/pbin/ci.yml?branch=main&label=FreeBSD%20aarch64)](https://github.com/watchthelight/pbin/actions/workflows/ci.yml) | [![NetBSD x86_64](https://img.shields.io/github/actions/workflow/status/watchthelight/pbin/ci.yml?branch=main&label=NetBSD%20x86_64)](https://github.com/watchthelight/pbin/actions/workflows/ci.yml) | [![OpenBSD x86_64](https://img.shields.io/github/actions/workflow/status/watchthelight/pbin/ci.yml?branch=main&label=OpenBSD%20x86_64)](https://github.com/watchthelight/pbin/actions/workflows/ci.yml) | [![Linux i686](https://img.shields.io/github/actions/workflow/status/watchthelight/pbin/ci.yml?branch=main&label=Linux%20i686)](https://github.com/watchthelight/pbin/actions/workflows/ci.yml) |
| [![Android aarch64](https://img.shields.io/github/actions/workflow/status/watchthelight/pbin/ci.yml?branch=main&label=Android%20aarch64)](https://github.com/watchthelight/pbin/actions/workflows/ci.yml) | [![Android armv7](https://img.shields.io/github/actions/workflow/status/watchthelight/pbin/ci.yml?branch=main&label=Android%20armv7)](https://github.com/watchthelight/pbin/actions/workflows/ci.yml) | [![Android x86_64](https://img.shields.io/github/actions/workflow/status/watchthelight/pbin/ci.yml?branch=main&label=Android%20x86_64)](https://github.com/watchthelight/pbin/actions/workflows/ci.yml) | [![iOS aarch64](https://img.shields.io/github/actions/workflow/status/watchthelight/pbin/ci.yml?branch=main&label=iOS%20aarch64)](https://github.com/watchthelight/pbin/actions/workflows/ci.yml) | [![WASI wasm32](https://img.shields.io/github/actions/workflow/status/watchthelight/pbin/ci.yml?branch=main&label=WASI%20wasm32)](https://github.com/watchthelight/pbin/actions/workflows/ci.yml) |
| [![Linux mips64](https://img.shields.io/github/actions/workflow/status/watchthelight/pbin/ci.yml?branch=main&label=Linux%20mips64)](https://github.com/watchthelight/pbin/actions/workflows/ci.yml) | [![Linux loongarch64](https://img.shields.io/github/actions/workflow/status/watchthelight/pbin/ci.yml?branch=main&label=Linux%20loongarch64)](https://github.com/watchthelight/pbin/actions/workflows/ci.yml) | [![Multi-Platform](https://img.shields.io/github/actions/workflow/status/watchthelight/pbin/ci.yml?branch=main&label=Multi-Platform)](https://github.com/watchthelight/pbin/actions/workflows/ci.yml) | | |

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
