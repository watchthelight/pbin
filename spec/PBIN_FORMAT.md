# PBIN Format Specification

**Version:** 1.0
**Status:** Draft
**Last Updated:** 2024

## Overview

PBIN (Polyglot Binary) is a file format for distributing cross-platform executables as a single file. A PBIN file is simultaneously:

1. A valid POSIX shell script
2. A valid Windows batch file
3. A container for compressed native executables

When executed, the polyglot stub detects the current platform, extracts the appropriate binary, and runs it.

## File Structure

```
┌─────────────────────────────────────────────────────────────┐
│ POLYGLOT STUB                                               │
│ (Valid as both shell script and batch file)                 │
│ Size: Variable, typically 2-4 KB                            │
├─────────────────────────────────────────────────────────────┤
│ PAYLOAD MARKER                                              │
│ "__PBIN_PAYLOAD__" (17 bytes)                               │
├─────────────────────────────────────────────────────────────┤
│ PBIN HEADER (Fixed: 64 bytes)                               │
├─────────────────────────────────────────────────────────────┤
│ MANIFEST (Variable length, JSON)                            │
├─────────────────────────────────────────────────────────────┤
│ BINARY PAYLOADS (Compressed)                                │
│ - One per target platform                                   │
└─────────────────────────────────────────────────────────────┘
```

## Polyglot Stub

The stub must execute correctly as:
- POSIX shell script (sh, bash, dash, zsh)
- Windows batch file (cmd.exe)

### Polyglot Technique

The stub exploits differences in how shell and batch interpret certain constructs:

```
:; # Batch sees a label `:`, shell sees `:; #` (no-op + comment)
:; exec sh -c '...'   # Shell executes, batch ignores (it's a label)
@echo off             # Batch executes, shell sees @ as command (errors quietly)
```

### Stub Responsibilities

1. Detect current OS (Linux, macOS, Windows)
2. Detect current architecture (x86_64, aarch64, riscv64)
3. Locate the payload marker in self
4. Parse the manifest to find correct binary offset
5. Extract binary to temporary location
6. Decompress if needed
7. Set executable permissions (Unix)
8. Execute with original arguments
9. Clean up temporary files
10. Exit with child's exit code

### Stub Size Target

The stub should be under 4KB to minimize overhead.

## Payload Marker

The literal ASCII string `__PBIN_PAYLOAD__` (17 bytes) marks the end of the polyglot stub and the beginning of the binary payload section. This allows the stub to locate the payload using simple string search.

## PBIN Header

Fixed 64-byte header immediately following the payload marker.

| Offset | Size | Field | Description |
|--------|------|-------|-------------|
| 0 | 4 | magic | ASCII "PBIN" (0x50 0x42 0x49 0x4E) |
| 4 | 2 | version | Format version (little-endian, currently 1) |
| 6 | 1 | compression | Compression type (0=none, 1=zstd, 2=lz4) |
| 7 | 1 | entry_count | Number of binary entries (max 255) |
| 8 | 4 | manifest_size | Size of JSON manifest in bytes (little-endian) |
| 12 | 4 | flags | Reserved flags (must be 0) |
| 16 | 48 | reserved | Reserved for future use (must be 0) |

Total: 64 bytes

## Manifest

JSON document following the header. Size specified in header's `manifest_size` field.

### Schema

```json
{
  "name": "string",
  "version": "string",
  "entries": [
    {
      "target": "string",
      "offset": number,
      "compressed_size": number,
      "uncompressed_size": number,
      "checksum": "string"
    }
  ]
}
```

### Fields

- **name**: Application name (e.g., "hello")
- **version**: Application version (e.g., "1.0.0")
- **entries**: Array of binary entries

### Entry Fields

- **target**: Target platform identifier (see Target Identifiers)
- **offset**: Byte offset from start of file to compressed binary data
- **compressed_size**: Size of compressed data in bytes
- **uncompressed_size**: Size of uncompressed binary in bytes
- **checksum**: BLAKE3 hash of uncompressed binary (64 hex characters)

## Target Identifiers

| Identifier | OS | Architecture | Rust Triple |
|------------|-----|--------------|-------------|
| linux-x86_64 | Linux | x86-64 | x86_64-unknown-linux-gnu |
| linux-aarch64 | Linux | ARM64 | aarch64-unknown-linux-gnu |
| linux-riscv64 | Linux | RISC-V 64 | riscv64gc-unknown-linux-gnu |
| darwin-x86_64 | macOS | x86-64 | x86_64-apple-darwin |
| darwin-aarch64 | macOS | ARM64 | aarch64-apple-darwin |
| windows-x86_64 | Windows | x86-64 | x86_64-pc-windows-msvc |
| windows-aarch64 | Windows | ARM64 | aarch64-pc-windows-msvc |

## Compression

### Supported Algorithms

| ID | Algorithm | Notes |
|----|-----------|-------|
| 0 | None | Raw binary, no compression |
| 1 | Zstandard | Recommended, best ratio |
| 2 | LZ4 | Faster decompression |

### Compression Level

For Zstandard, level 19 is recommended for distribution builds.

## Binary Payloads

Compressed (or raw) binaries are concatenated after the manifest. Each entry's `offset` field provides the absolute file offset to its data.

Binaries are stored in the order they appear in the manifest.

## Execution Flow

### Unix (Shell Path)

```
1. Script starts with #!/bin/sh or is executed with sh
2. Detect OS: uname -s → Linux | Darwin
3. Detect arch: uname -m → x86_64 | aarch64 | arm64 | riscv64
4. Find payload offset: grep -abo "__PBIN_PAYLOAD__" "$0"
5. Read header (64 bytes after marker)
6. Parse manifest (JSON)
7. Find entry matching current platform
8. Extract to temp file: dd if="$0" bs=1 skip=$offset count=$size
9. Decompress: zstd -d or similar
10. chmod +x
11. Execute with "$@"
12. Capture exit code
13. Clean up temp files
14. Exit with captured code
```

### Windows (Batch Path)

```
1. Script runs as batch file
2. Detect arch: %PROCESSOR_ARCHITECTURE%
3. Use PowerShell or certutil for extraction
4. Decompress using bundled tool or PowerShell
5. Execute extracted .exe
6. Clean up
7. Exit with %ERRORLEVEL%
```

## Temporary Files

### Location

- **Unix**: `$TMPDIR` or `/tmp`, subdirectory `pbin-XXXXXX`
- **Windows**: `%TEMP%`, subdirectory `pbin-XXXXXX`

### Naming

Extracted binary: `{original_name}` or `{original_name}.exe`

### Cleanup

Temporary files MUST be cleaned up on:
- Normal exit
- Signal interruption (SIGINT, SIGTERM on Unix)

Use trap handlers on Unix; Windows batch has limited signal handling.

## Security Considerations

### Checksum Verification

Implementations SHOULD verify BLAKE3 checksums before execution.

### Temp Directory Permissions

Unix: Create temp directory with mode 0700.

### Path Handling

All paths must be quoted to handle spaces correctly.

## File Extension

The recommended extension is `.pbin`. Files should be marked executable on Unix systems.

## MIME Type

Suggested: `application/x-pbin`

## Versioning

The format version in the header allows for future extensions. Readers should reject versions they don't understand.

## Example

Minimal PBIN with single Linux x86_64 binary:

```
#!/bin/sh
:; # PBIN polyglot stub
:; SELF="$0"
:; ... (stub implementation)
:; exit $?
@echo off
rem Windows stub
exit /b %ERRORLEVEL%
__PBIN_PAYLOAD__
PBIN[version=1][compression=1][entries=1][manifest_size=XXX]
{"name":"hello","version":"1.0.0","entries":[{"target":"linux-x86_64",...}]}
[compressed binary data]
```

## Reference Implementation

See `pbin-core` crate for Rust implementation.
