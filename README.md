# CacheHarvest

A lightweight Rust utility for exporting recoverable image assets from Google Chrome cache on Windows.

## Features

- Recursively scans Chrome cache files
- Supports cache path fallback (`<profile>/Cache`, then `<profile>/Network/Cache`)
- Detects file types from binary signatures (`infer`)
- Exports only `image/*` files (PNG, JPG, WebP, GIF, etc.)
- Skips duplicate binaries by default
- Uses safe read-only access to browser cache data
- Supports custom output directory and profile selection

## Requirements

- Rust stable toolchain
- Windows 10/11

## Build

```bash
cargo build --release
```

Release binary location:

```bash
target/release/cacheharvest.exe
```

## Usage

Default mode (exports to Downloads/cacheharvest_export):

```bash
cacheharvest.exe
```

Custom output folder:

```bash
cacheharvest.exe "C:\\MyCustomFolder"
```

Additional flags:

```bash
cacheharvest.exe --profile "Default" --min-size 128 --keep-duplicates
```

If the export directory cannot be created, CacheHarvest exits with a descriptive error.

## Notes

- Chrome should be closed for best extraction results.
- Only files currently present in cache can be exported.
- CacheHarvest does not parse Chrome cache index metadata; it scans raw file contents.
- No network calls are made.
