# PBIN - Polyglot Binary Format

[![Linux x86_64](https://github.com/watchthelight/pbin/actions/workflows/ci.yml/badge.svg?branch=main&event=push&job=test-linux-x86_64)](https://github.com/watchthelight/pbin/actions/workflows/ci.yml)
[![Linux ARM64](https://github.com/watchthelight/pbin/actions/workflows/ci.yml/badge.svg?branch=main&event=push&job=test-linux-arm64)](https://github.com/watchthelight/pbin/actions/workflows/ci.yml)
[![macOS x86_64](https://github.com/watchthelight/pbin/actions/workflows/ci.yml/badge.svg?branch=main&event=push&job=test-macos-x86_64)](https://github.com/watchthelight/pbin/actions/workflows/ci.yml)
[![macOS ARM64](https://github.com/watchthelight/pbin/actions/workflows/ci.yml/badge.svg?branch=main&event=push&job=test-macos-arm64)](https://github.com/watchthelight/pbin/actions/workflows/ci.yml)
[![Windows x86_64](https://github.com/watchthelight/pbin/actions/workflows/ci.yml/badge.svg?branch=main&event=push&job=test-windows-x86_64)](https://github.com/watchthelight/pbin/actions/workflows/ci.yml)
[![Rust Tests](https://github.com/watchthelight/pbin/actions/workflows/ci.yml/badge.svg?branch=main&event=push&job=test-rust)](https://github.com/watchthelight/pbin/actions/workflows/ci.yml)

A revolutionary file format and toolchain that enables single-file, cross-platform executables. A `.pbin` file runs on any supported system with just `./file.pbin` (Unix) or double-click (Windows) - no installation, no runtime dependencies, no setup.

## Supported Platforms

| OS      | Architectures              | Status |
|---------|----------------------------|--------|
| Linux   | x86_64, aarch64            | ✅ |
| macOS   | x86_64, aarch64 (Apple Silicon) | ✅ |
| Windows | x86_64                     | ✅ |

## How It Works

PBIN uses a polyglot header strategy - a file that is simultaneously:
- A valid POSIX shell script
- A valid Windows batch file
- A container for compressed native executables

When executed, the polyglot stub:
1. Detects the current OS and architecture
2. Locates the correct embedded binary
3. Extracts it to a temp location
4. Executes it with all original arguments
5. Cleans up on exit

## Quick Start

```bash
# Create a .pbin from pre-built binaries
pbin-pack \
  --name myapp \
  --linux-x86_64 ./target/x86_64-unknown-linux-gnu/release/myapp \
  --darwin-aarch64 ./target/aarch64-apple-darwin/release/myapp \
  --windows-x86_64 ./target/x86_64-pc-windows-msvc/release/myapp.exe \
  --output myapp.pbin

# Run it anywhere!
./myapp.pbin        # Linux/macOS
.\myapp.pbin        # Windows
```

## Building from Source

```bash
# Build all tools
cargo build --release

# Build test payload for available targets
./scripts/build-all-targets.sh

# Create example .pbin
./target/release/pbin-pack \
  --name hello \
  --darwin-aarch64 ./target/pbin-payloads/hello-darwin-aarch64 \
  --output hello.pbin
```

## Project Structure

```
pbin/
├── spec/                    # Format specification
├── crates/
│   ├── pbin-core/          # Format parsing, manifest handling
│   ├── pbin-pack/          # CLI: pack binaries into .pbin
│   ├── pbin-stub/          # Polyglot stub generator
│   └── pbin-unpack/        # CLI: extract/inspect .pbin files
├── stubs/                   # Stub templates
├── test-payload/           # Test programs
└── scripts/                # Build and CI scripts
```

## License

MIT License - see [LICENSE](LICENSE) for details.
