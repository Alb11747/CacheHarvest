use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::errors::AppError;

#[derive(Debug, Default, Clone)]
pub struct ExportStats {
    pub scanned_files: usize,
    pub exported_files: usize,
    pub skipped_not_image: usize,
    pub skipped_invalid_cache: usize,
    pub skipped_duplicate: usize,
    pub skipped_read_error: usize,
    pub skipped_write_error: usize,
}

#[derive(Debug, Clone)]
pub struct ExportOptions {
    pub dedupe: bool,
}

fn digest(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn record(
    manifest: &mut fs::File,
    source: &Path,
    key: Option<&str>,
    target: &Path,
    mime: &str,
    hash: &str,
    duplicate: bool,
) -> io::Result<()> {
    // Partition keys can contain multiple origins. Only expose a URL when the
    // entire key is plainly one; preserve every other cache key verbatim.
    let url = key.filter(|key| {
        (key.starts_with("https://") || key.starts_with("http://"))
            && !key.chars().any(char::is_whitespace)
    });
    let mut line = serde_json::to_vec(&serde_json::json!({
        "source_path": source.to_string_lossy(),
        "cache_key": key,
        "url": url,
        "output_path": target.to_string_lossy(),
        "mime": mime,
        "sha256": hash,
        "duplicate": duplicate,
    }))?;
    line.push(b'\n');
    manifest.write_all(&line)?;
    manifest.flush()
}

pub fn export_images(
    source_files: &[PathBuf],
    output_dir: &Path,
    options: &ExportOptions,
) -> Result<ExportStats, AppError> {
    let mut stats = ExportStats::default();
    let mut counter: usize = 1;
    let mut seen_hashes = HashMap::<String, PathBuf>::new();

    fs::create_dir_all(output_dir).map_err(|source| AppError::OutputDirectoryCreate {
        path: output_dir.display().to_string(),
        source,
    })?;

    // Hash actual files instead of trusting a potentially stale manifest: users
    // may have renamed, modified, or removed previous exports between runs.
    if options.dedupe {
        for entry in fs::read_dir(output_dir)? {
            let entry = entry?;
            if !entry.file_type()?.is_file() || entry.file_name() == "manifest.jsonl" {
                continue;
            }
            let bytes = fs::read(entry.path())?;
            if infer::get(&bytes).is_some_and(|kind| kind.mime_type().starts_with("image/")) {
                seen_hashes.entry(digest(&bytes)).or_insert(entry.path());
            }
        }
    }
    let mut manifest = OpenOptions::new()
        .create(true)
        .append(true)
        .open(output_dir.join("manifest.jsonl"))?;

    for file in source_files {
        stats.scanned_files += 1;
        let bytes = match fs::read(file) {
            Ok(value) => value,
            Err(error) => {
                eprintln!("Unable to read '{}': {error}", file.display());
                stats.skipped_read_error += 1;
                continue;
            }
        };
        let payload = match crate::cache_entry::extract(&bytes) {
            Ok(payload) => payload,
            Err(error) => {
                eprintln!("Invalid cache entry '{}': {error}", file.display());
                stats.skipped_invalid_cache += 1;
                continue;
            }
        };
        let kind = match infer::get(payload.data) {
            Some(kind) if kind.mime_type().starts_with("image/") => kind,
            _ => {
                stats.skipped_not_image += 1;
                continue;
            }
        };
        let hash = digest(payload.data);
        if let Some(target) = seen_hashes.get(&hash).filter(|_| options.dedupe) {
            record(
                &mut manifest,
                file,
                payload.key.as_deref(),
                target,
                kind.mime_type(),
                &hash,
                true,
            )?;
            stats.skipped_duplicate += 1;
            continue;
        }

        // Exclusive creation protects every pre-existing file, including files
        // created by another process after the directory was scanned.
        let created = loop {
            let target = output_dir.join(format!("{counter:04}.{}", kind.extension()));
            counter += 1;
            match OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&target)
            {
                Ok(output) => break Some((target, output)),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => {
                    eprintln!("Unable to create '{}': {error}", target.display());
                    stats.skipped_write_error += 1;
                    break None;
                }
            }
        };
        let Some((target, mut output)) = created else {
            continue;
        };
        if let Err(error) = output.write_all(payload.data).and_then(|_| output.flush()) {
            eprintln!("Unable to write '{}': {error}", target.display());
            drop(output);
            if let Err(error) = fs::remove_file(&target) {
                eprintln!(
                    "Unable to remove incomplete export '{}': {error}",
                    target.display()
                );
            }
            stats.skipped_write_error += 1;
            continue;
        }
        drop(output);
        record(
            &mut manifest,
            file,
            payload.key.as_deref(),
            &target,
            kind.mime_type(),
            &hash,
            false,
        )?;
        stats.exported_files += 1;
        if options.dedupe {
            seen_hashes.insert(hash, target);
        }
    }

    Ok(stats)
}
