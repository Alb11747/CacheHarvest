use crate::errors::AppError;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct ScanOptions {
    pub min_size_bytes: u64,
}

pub fn collect_cache_files(
    cache_dir: &Path,
    options: &ScanOptions,
) -> Result<Vec<PathBuf>, AppError> {
    let mut files = Vec::new();
    for entry in WalkDir::new(cache_dir).follow_links(false) {
        let entry = entry.map_err(std::io::Error::other)?;
        if entry.file_type().is_file()
            && entry.metadata().map_err(std::io::Error::other)?.len() >= options.min_size_bytes
        {
            files.push(entry.path().to_path_buf());
        }
    }
    files.sort();
    Ok(files)
}
