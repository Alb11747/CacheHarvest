use std::path::{Path, PathBuf};

use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct ScanOptions {
    pub min_size_bytes: u64,
}

pub fn collect_cache_files(cache_dir: &Path, options: &ScanOptions) -> Vec<PathBuf> {
    WalkDir::new(cache_dir)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter_map(|entry| {
            let metadata = entry.metadata().ok()?;
            if metadata.len() < options.min_size_bytes {
                return None;
            }
            Some(entry.path().to_path_buf())
        })
        .collect()
}
