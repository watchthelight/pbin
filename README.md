# PBIN - Polyglot Binary Format

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/github/actions/workflow/status/watchthelight/pbin/tests.yml?branch=main&label=Tests)](https://github.com/watchthelight/pbin/actions/workflows/tests.yml)

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
| [![Linux x86_64](https://img.shields.io/github/actions/workflow/status/watchthelight/pbin/linux-x86_64.yml?branch=main&label=Linux%20x86_64)](https://github.com/watchthelight/pbin/actions/workflows/linux-x86_64.yml) | [![Linux aarch64](https://img.shields.io/github/actions/workflow/status/watchthelight/pbin/linux-aarch64.yml?branch=main&label=Linux%20aarch64)](https://github.com/watchthelight/pbin/actions/workflows/linux-aarch64.yml) | [![macOS x86_64](https://img.shields.io/github/actions/workflow/status/watchthelight/pbin/macos-x86_64.yml?branch=main&label=macOS%20x86_64)](https://github.com/watchthelight/pbin/actions/workflows/macos-x86_64.yml) | [![macOS aarch64](https://img.shields.io/github/actions/workflow/status/watchthelight/pbin/macos-aarch64.yml?branch=main&label=macOS%20aarch64)](https://github.com/watchthelight/pbin/actions/workflows/macos-aarch64.yml) | [![Windows x86_64](https://img.shields.io/github/actions/workflow/status/watchthelight/pbin/windows-x86_64.yml?branch=main&label=Windows%20x86_64)](https://github.com/watchthelight/pbin/actions/workflows/windows-x86_64.yml) | [![Windows aarch64](https://img.shields.io/github/actions/workflow/status/watchthelight/pbin/windows-aarch64.yml?branch=main&label=Windows%20aarch64)](https://github.com/watchthelight/pbin/actions/workflows/windows-aarch64.yml) |

### Standard Tier
| | | | | | | |
|:-:|:-:|:-:|:-:|:-:|:-:|:-:|
| [![Linux musl x86_64](https://img.shields.io/github/actions/workflow/status/watchthelight/pbin/linux-musl-x86_64.yml?branch=main&label=Linux%20musl%20x86_64)](https://github.com/watchthelight/pbin/actions/workflows/linux-musl-x86_64.yml) | [![Linux musl aarch64](https://img.shields.io/github/actions/workflow/status/watchthelight/pbin/linux-musl-aarch64.yml?branch=main&label=Linux%20musl%20aarch64)](https://github.com/watchthelight/pbin/actions/workflows/linux-musl-aarch64.yml) | [![Linux armv7](https://img.shields.io/github/actions/workflow/status/watchthelight/pbin/linux-armv7.yml?branch=main&label=Linux%20armv7)](https://github.com/watchthelight/pbin/actions/workflows/linux-armv7.yml) | [![Linux riscv64](https://img.shields.io/github/actions/workflow/status/watchthelight/pbin/linux-riscv64.yml?branch=main&label=Linux%20riscv64)](https://github.com/watchthelight/pbin/actions/workflows/linux-riscv64.yml) | [![Linux ppc64le](https://img.shields.io/github/actions/workflow/status/watchthelight/pbin/linux-ppc64le.yml?branch=main&label=Linux%20ppc64le)](https://github.com/watchthelight/pbin/actions/workflows/linux-ppc64le.yml) | [![Linux s390x](https://img.shields.io/github/actions/workflow/status/watchthelight/pbin/linux-s390x.yml?branch=main&label=Linux%20s390x)](https://github.com/watchthelight/pbin/actions/workflows/linux-s390x.yml) | [![Windows x86](https://img.shields.io/github/actions/workflow/status/watchthelight/pbin/windows-x86.yml?branch=main&label=Windows%20x86)](https://github.com/watchthelight/pbin/actions/workflows/windows-x86.yml) |

### Extended Tier
| | | | | |
|:-:|:-:|:-:|:-:|:-:|
| [![FreeBSD x86_64](https://img.shields.io/github/actions/workflow/status/watchthelight/pbin/freebsd-x86_64.yml?branch=main&label=FreeBSD%20x86_64)](https://github.com/watchthelight/pbin/actions/workflows/freebsd-x86_64.yml) | [![FreeBSD aarch64](https://img.shields.io/github/actions/workflow/status/watchthelight/pbin/freebsd-aarch64.yml?branch=main&label=FreeBSD%20aarch64)](https://github.com/watchthelight/pbin/actions/workflows/freebsd-aarch64.yml) | [![NetBSD x86_64](https://img.shields.io/github/actions/workflow/status/watchthelight/pbin/netbsd-x86_64.yml?branch=main&label=NetBSD%20x86_64)](https://github.com/watchthelight/pbin/actions/workflows/netbsd-x86_64.yml) | [![OpenBSD x86_64](https://img.shields.io/github/actions/workflow/status/watchthelight/pbin/openbsd-x86_64.yml?branch=main&label=OpenBSD%20x86_64)](https://github.com/watchthelight/pbin/actions/workflows/openbsd-x86_64.yml) | [![Linux i686](https://img.shields.io/github/actions/workflow/status/watchthelight/pbin/linux-i686.yml?branch=main&label=Linux%20i686)](https://github.com/watchthelight/pbin/actions/workflows/linux-i686.yml) |
| [![Android aarch64](https://img.shields.io/github/actions/workflow/status/watchthelight/pbin/android-aarch64.yml?branch=main&label=Android%20aarch64)](https://github.com/watchthelight/pbin/actions/workflows/android-aarch64.yml) | [![Android armv7](https://img.shields.io/github/actions/workflow/status/watchthelight/pbin/android-armv7.yml?branch=main&label=Android%20armv7)](https://github.com/watchthelight/pbin/actions/workflows/android-armv7.yml) | [![Android x86_64](https://img.shields.io/github/actions/workflow/status/watchthelight/pbin/android-x86_64.yml?branch=main&label=Android%20x86_64)](https://github.com/watchthelight/pbin/actions/workflows/android-x86_64.yml) | [![iOS aarch64](https://img.shields.io/github/actions/workflow/status/watchthelight/pbin/ios-aarch64.yml?branch=main&label=iOS%20aarch64)](https://github.com/watchthelight/pbin/actions/workflows/ios-aarch64.yml) | [![WASI wasm32](https://img.shields.io/github/actions/workflow/status/watchthelight/pbin/wasi-wasm32.yml?branch=main&label=WASI%20wasm32)](https://github.com/watchthelight/pbin/actions/workflows/wasi-wasm32.yml) |
| [![Linux mips64](https://img.shields.io/github/actions/workflow/status/watchthelight/pbin/linux-mips64.yml?branch=main&label=Linux%20mips64)](https://github.com/watchthelight/pbin/actions/workflows/linux-mips64.yml) | [![Linux loongarch64](https://img.shields.io/github/actions/workflow/status/watchthelight/pbin/linux-loongarch64.yml?branch=main&label=Linux%20loongarch64)](https://github.com/watchthelight/pbin/actions/workflows/linux-loongarch64.yml) | | | |

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
