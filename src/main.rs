use std::env;
use std::path::PathBuf;

use clap::Parser;

use cacheharvest::cache_scanner::{collect_cache_files, ScanOptions};
use cacheharvest::errors::AppError;
use cacheharvest::exporter::{export_images, ExportOptions};
use cacheharvest::paths::{chrome_cache_candidates, existing_chrome_cache_dirs};

#[derive(Debug, Parser)]
#[command(name = "cacheharvest")]
#[command(version)]
#[command(about = "Export recoverable image assets from Chrome cache")]
struct Cli {
    #[arg(help = "Optional output directory. Defaults to Downloads/cacheharvest_export")]
    output_dir: Option<PathBuf>,

    #[arg(long, default_value = "Default", help = "Chrome profile directory name")]
    profile: String,

    #[arg(long, default_value_t = 128, help = "Minimum cache file size in bytes")]
    min_size: u64,

    #[arg(
        long,
        default_value_t = false,
        help = "Keep duplicate image binaries instead of skipping duplicates"
    )]
    keep_duplicates: bool,
}

fn chrome_cache_dirs(profile: &str) -> Result<Vec<PathBuf>, AppError> {
    let local = env::var("LOCALAPPDATA").map_err(|_| AppError::MissingLocalAppData)?;
    let local_path = PathBuf::from(local);
    let candidates = chrome_cache_candidates(&local_path, profile);
    let existing = existing_chrome_cache_dirs(&local_path, profile);

    if existing.is_empty() {
        return Err(AppError::MissingChromeCacheDirectories(
            candidates
                .iter()
                .map(|path| path.display().to_string())
                .collect(),
        ));
    }

    Ok(existing)
}

fn default_export_dir() -> Result<PathBuf, AppError> {
    if let Some(downloads) = dirs::download_dir() {
        return Ok(downloads.join("cacheharvest_export"));
    }

    let home = dirs::home_dir().ok_or(AppError::MissingHomeDirectory)?;
    Ok(home.join("Downloads").join("cacheharvest_export"))
}

fn run() -> Result<(), AppError> {
    let args = Cli::parse();

    let cache_dirs = chrome_cache_dirs(&args.profile)?;
    let output_dir = match args.output_dir {
        Some(path) => path,
        None => default_export_dir()?,
    };

    let mut files = Vec::new();
    for cache_dir in &cache_dirs {
        let mut entries = collect_cache_files(
            cache_dir,
            &ScanOptions {
                min_size_bytes: args.min_size,
            },
        );
        files.append(&mut entries);
    }

    let stats = export_images(
        &files,
        &output_dir,
        &ExportOptions {
            dedupe: !args.keep_duplicates,
        },
    )?;

    println!("CacheHarvest completed.");
    println!("Cache directories:");
    for cache_dir in &cache_dirs {
        println!("  - {}", cache_dir.display());
    }
    println!("Export directory: {}", output_dir.display());
    println!("Scanned         : {}", stats.scanned_files);
    println!("Exported        : {}", stats.exported_files);
    println!("Skipped(non-img): {}", stats.skipped_not_image);
    println!("Skipped(dup)    : {}", stats.skipped_duplicate);
    println!("Read errors     : {}", stats.skipped_read_error);
    println!("Write errors    : {}", stats.skipped_write_error);

    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}
