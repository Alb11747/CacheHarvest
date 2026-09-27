# CacheHarvest

A Windows command-line utility for recovering image response bodies from Chrome cache files. Browser data is read only; no network requests are made by the utility.

## Supported recovery

- Chromium Simple Cache **v5 combined stream 0/1 entries**, with bounded parsing, optional SHA-256 key verification, and stream CRC32 verification when present.
- Standalone image files identified by binary signatures (PNG, JPEG, WebP, GIF, and other `infer` image formats).
- Chrome profile discovery under `Cache` and `Network/Cache`, or an explicit copied cache directory.
- SHA-256 deduplication against both the current run and existing image files in the output directory.
- Exclusive file creation: existing exports are never overwritten. `--keep-duplicates` creates additional uniquely numbered files.
- Append-only `manifest.jsonl` recording source paths, cache keys, detected URLs, output paths, MIME types and hashes, including duplicate associations.

## Build and test

Requires a current stable Rust toolchain. On Windows:

```powershell
cargo test --locked --all-targets --all-features
cargo build --locked --release
```

The executable is `target/release/cacheharvest.exe`. GitHub Actions builds and uploads a Windows executable for pushes and pull requests; pushing a version tag such as `v0.1.1` publishes a release with the executable attached.

## Usage

```powershell
# Default Chrome profile; exports to Downloads/cacheharvest_export
cacheharvest.exe

# Explicit output and profile
cacheharvest.exe "C:\RecoveredImages" --profile "Profile 1"

# Recover from a copied cache, without requiring LOCALAPPDATA
cacheharvest.exe "C:\RecoveredImages" --cache-dir "D:\CacheCopy"

cacheharvest.exe --min-size 128 --keep-duplicates
```

`--min-size` filters source file sizes, not extracted body sizes. Close Chrome or use a cache copy for consistent results. Use an output directory outside the source cache and run one exporter per output directory at a time.

The manifest preserves the full cache key because partitioned keys may contain more than a URL. A URL is recorded only when an HTTP(S) URL can be identified. Original filenames are not used as output paths. The manifest can contain private browsing URLs; treat it like the cache itself.

## Limits and error reporting

This is not a universal browser cache reader. Legacy blockfile caches, sparse entries, separate stream-2 files, compressed HTTP response decoding, and unsupported Simple Cache versions are not recovered. Index and unrelated files are skipped. Image detection is signature-based, not a guarantee that an image is complete or decodable. Only data still present on disk is recoverable.

Malformed recognized entries are counted separately from non-images. Read/write failures are reported, and the CLI exits unsuccessfully when recovery encounters these errors or invalid recognized entries. Traversal and manifest failures also return an error. Successfully written images remain available after a partial failure.

Tests include complete PNG data verified by decoding during fixture creation, format-faithful Simple Cache files with metadata, CRCs and key digests, corruption and truncation cases, repeat exports and collision protection. The fixtures are synthetic and contain no personal browser data; passing these tests is not proof of recovery from every installed Chrome version.

Format reference: [Chromium Simple Cache entry layout](https://chromium.googlesource.com/chromium/src/+/main/net/disk_cache/simple/simple_entry_format.h).
