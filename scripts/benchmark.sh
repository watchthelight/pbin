#!/bin/bash
# PBIN Compression Benchmark
# Compares different compression settings

set -e

echo "=== PBIN Compression Benchmark ==="
echo ""

# Build tools
echo "Building release binaries..."
cargo build --release -p pbin-pack -p hello 2>/dev/null

PBIN_PACK="./target/release/pbin-pack"
HELLO="./target/release/hello"
OUTDIR="/tmp/pbin-benchmark"

rm -rf "$OUTDIR"
mkdir -p "$OUTDIR"

ORIGINAL_SIZE=$(stat -f%z "$HELLO" 2>/dev/null || stat -c%s "$HELLO")
echo "Original binary size: $ORIGINAL_SIZE bytes"
echo ""

# Function to benchmark a compression setting
benchmark() {
    local name="$1"
    shift
    local start=$(date +%s%N 2>/dev/null || date +%s)

    $PBIN_PACK --name hello --version 1.0.0 \
        --darwin-aarch64 "$HELLO" \
        --output "$OUTDIR/$name.pbin" \
        "$@" 2>/dev/null

    local end=$(date +%s%N 2>/dev/null || date +%s)
    local size=$(stat -f%z "$OUTDIR/$name.pbin" 2>/dev/null || stat -c%s "$OUTDIR/$name.pbin")
    local ratio=$(echo "scale=1; $size * 100 / $ORIGINAL_SIZE" | bc)
    local savings=$(echo "scale=1; 100 - $ratio" | bc)

    printf "%-25s %10s bytes  %5s%%  (saved %s%%)\n" "$name:" "$size" "$ratio" "$savings"
}

echo "Compression Benchmarks:"
echo "------------------------"

benchmark "no-compress" --no-compress
benchmark "fast" --compress fast --no-bcj
benchmark "fast+bcj" --compress fast
benchmark "balanced" --compress balanced --no-bcj
benchmark "balanced+bcj" --compress balanced
benchmark "maximum" --compress maximum --no-bcj
benchmark "maximum+bcj" --compress maximum

echo ""
echo "Testing execution of compressed PBIN..."
chmod +x "$OUTDIR/balanced.pbin"
echo "yes" | "$OUTDIR/balanced.pbin" 2>/dev/null | head -1

echo ""
echo "Benchmark complete! Files in $OUTDIR"
