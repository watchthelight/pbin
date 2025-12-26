#!/usr/bin/env bash
# build-all-targets.sh
#
# Cross-compiles the test payload for all supported targets.
# Run from the repository root.
#
# Prerequisites:
# - Rust toolchain with cross-compilation targets installed
# - For Linux targets: `cross` (https://github.com/cross-rs/cross) or Docker
# - For Windows targets: `cargo-xwin` and xwin SDK, or MSVC toolchain
# - For macOS: Xcode command line tools (native), or osxcross for cross-compile

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
OUTPUT_DIR="$PROJECT_ROOT/target/pbin-payloads"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Get PBIN target name from Rust triple
get_pbin_name() {
    case "$1" in
        x86_64-unknown-linux-gnu) echo "linux-x86_64" ;;
        aarch64-unknown-linux-gnu) echo "linux-aarch64" ;;
        riscv64gc-unknown-linux-gnu) echo "linux-riscv64" ;;
        x86_64-apple-darwin) echo "darwin-x86_64" ;;
        aarch64-apple-darwin) echo "darwin-aarch64" ;;
        x86_64-pc-windows-msvc) echo "windows-x86_64" ;;
        aarch64-pc-windows-msvc) echo "windows-aarch64" ;;
        *) echo "" ;;
    esac
}

# Get binary extension for target
get_extension() {
    case "$1" in
        *-windows-*) echo ".exe" ;;
        *) echo "" ;;
    esac
}

# Check if cross is usable (requires Docker)
cross_available() {
    command -v cross &> /dev/null && command -v docker &> /dev/null && docker info &> /dev/null
}

# Check if we can build a target
can_build_target() {
    local target=$1
    local os_name
    os_name="$(uname -s)"

    case "$target" in
        *-apple-darwin)
            # Can only build macOS targets on macOS
            [ "$os_name" = "Darwin" ]
            ;;
        *-linux-*)
            # Can build natively on Linux, or with cross (requires Docker)
            [ "$os_name" = "Linux" ] || cross_available
            ;;
        *-windows-*)
            # Can build with cargo-xwin or on Windows
            [[ "$os_name" == *"MINGW"* ]] || [[ "$os_name" == *"MSYS"* ]] || command -v cargo-xwin &> /dev/null
            ;;
        *)
            return 1
            ;;
    esac
}

# Build a single target
build_target() {
    local target=$1
    local pbin_name
    local ext
    pbin_name=$(get_pbin_name "$target")
    ext=$(get_extension "$target")

    info "Building for $pbin_name ($target)..."

    # Ensure target is installed
    rustup target add "$target" 2>/dev/null || true

    local build_cmd=""
    local os_name
    os_name="$(uname -s)"

    case "$target" in
        *-linux-*)
            if [ "$os_name" = "Linux" ]; then
                build_cmd="cargo build --release --target $target -p hello"
            else
                build_cmd="cross build --release --target $target -p hello"
            fi
            ;;
        *-apple-darwin)
            build_cmd="cargo build --release --target $target -p hello"
            ;;
        *-windows-*)
            if command -v cargo-xwin &> /dev/null; then
                build_cmd="cargo xwin build --release --target $target -p hello"
            else
                build_cmd="cargo build --release --target $target -p hello"
            fi
            ;;
    esac

    if eval "$build_cmd"; then
        # Copy to output directory
        local src="$PROJECT_ROOT/target/$target/release/hello$ext"
        local dst="$OUTPUT_DIR/hello-$pbin_name$ext"

        if [ -f "$src" ]; then
            cp "$src" "$dst"
            local size
            size=$(wc -c < "$dst" | tr -d ' ')
            info "  -> $dst ($size bytes)"
            return 0
        else
            error "  Binary not found: $src"
            return 1
        fi
    else
        error "  Build failed for $target"
        return 1
    fi
}

main() {
    info "PBIN Test Payload Cross-Compilation"
    info "===================================="
    info ""

    # Create output directory
    mkdir -p "$OUTPUT_DIR"

    # Target list
    local targets="
        x86_64-unknown-linux-gnu
        aarch64-unknown-linux-gnu
        riscv64gc-unknown-linux-gnu
        x86_64-apple-darwin
        aarch64-apple-darwin
        x86_64-pc-windows-msvc
        aarch64-pc-windows-msvc
    "

    local built=0
    local failed=0
    local skipped=0

    for target in $targets; do
        if can_build_target "$target"; then
            if build_target "$target"; then
                built=$((built + 1))
            else
                failed=$((failed + 1))
            fi
        else
            warn "Skipping $target (tools not available)"
            skipped=$((skipped + 1))
        fi
    done

    info ""
    info "===================================="
    info "Summary: $built built, $failed failed, $skipped skipped"

    if [ "$built" -gt 0 ]; then
        info ""
        info "Built binaries:"
        ls -la "$OUTPUT_DIR"/
    fi

    if [ "$failed" -gt 0 ]; then
        exit 1
    fi
}

main "$@"
