use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

#[derive(Debug, Default, Clone)]
pub struct ExportStats {
    pub scanned_files: usize,
    pub exported_files: usize,
    pub skipped_not_image: usize,
    pub skipped_duplicate: usize,
    pub skipped_read_error: usize,
    pub skipped_write_error: usize,
}

#[derive(Debug, Clone)]
pub struct ExportOptions {
    pub dedupe: bool,
}

pub fn export_images(
    source_files: &[PathBuf],
    output_dir: &Path,
    options: &ExportOptions,
) -> ExportStats {
    let mut stats = ExportStats::default();
    let mut counter: usize = 1;
    let mut seen_hashes: HashSet<String> = HashSet::new();

    if let Err(_err) = fs::create_dir_all(output_dir) {
        return stats;
    }

    for file in source_files {
        stats.scanned_files += 1;

        let bytes = match fs::read(file) {
            Ok(value) => value,
            Err(_) => {
                stats.skipped_read_error += 1;
                continue;
            }
        };

        let kind = match infer::get(&bytes) {
            Some(value) => value,
            None => {
                stats.skipped_not_image += 1;
                continue;
            }
        };

        if !kind.mime_type().starts_with("image/") {
            stats.skipped_not_image += 1;
            continue;
        }

        if options.dedupe {
            let mut hasher = Sha256::new();
            hasher.update(&bytes);
            let digest = hex::encode(hasher.finalize());

            if seen_hashes.contains(&digest) {
                stats.skipped_duplicate += 1;
                continue;
            }

            seen_hashes.insert(digest);
        }

        let file_name = format!("{counter:04}.{}", kind.extension());
        let target = output_dir.join(file_name);

        match fs::File::create(target) {
            Ok(mut output_file) => {
                if output_file.write_all(&bytes).is_ok() {
                    stats.exported_files += 1;
                    counter += 1;
                } else {
                    stats.skipped_write_error += 1;
                }
            }
            Err(_) => {
                stats.skipped_write_error += 1;
            }
        }
    }

    stats
}
