use std::env;
use std::path::PathBuf;

use clap::Parser;

use cacheharvest::cache_scanner::{collect_cache_files, ScanOptions};
use cacheharvest::errors::AppError;
use cacheharvest::exporter::{export_images, ExportOptions};

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

fn chrome_cache_dir(profile: &str) -> Result<PathBuf, AppError> {
    let local = env::var("LOCALAPPDATA").map_err(|_| AppError::MissingLocalAppData)?;
    let path = PathBuf::from(local)
        .join("Google")
        .join("Chrome")
        .join("User Data")
        .join(profile)
        .join("Cache");

    if !path.exists() {
        return Err(AppError::MissingChromeCacheDirectory(
            path.display().to_string(),
        ));
    }

    Ok(path)
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

    let cache_dir = chrome_cache_dir(&args.profile)?;
    let output_dir = match args.output_dir {
        Some(path) => path,
        None => default_export_dir()?,
    };

    let files = collect_cache_files(
        &cache_dir,
        &ScanOptions {
            min_size_bytes: args.min_size,
        },
    );

    let stats = export_images(
        &files,
        &output_dir,
        &ExportOptions {
            dedupe: !args.keep_duplicates,
        },
    );

    println!("CacheHarvest completed.");
    println!("Cache directory : {}", cache_dir.display());
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
