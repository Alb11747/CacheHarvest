use std::fs;

use cacheharvest::cache_scanner::{collect_cache_files, ScanOptions};
use cacheharvest::exporter::{export_images, ExportOptions};
use cacheharvest::paths::{chrome_cache_candidates, existing_chrome_cache_dirs};

fn unique_test_dir(label: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "cacheharvest_test_{}_{}",
        label,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time")
            .as_nanos()
    ))
}

#[test]
fn exports_only_images_and_skips_duplicates() {
    let base = unique_test_dir("dedupe");

    let source_dir = base.join("source");
    let output_dir = base.join("output");
    fs::create_dir_all(&source_dir).expect("create source");

    let png = vec![
        0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, b'I', b'H',
        b'D', b'R',
    ];
    let not_image = b"hello text file".to_vec();

    let file_a = source_dir.join("a.bin");
    let file_b = source_dir.join("b.bin");
    let file_c = source_dir.join("c.bin");

    fs::write(&file_a, &png).expect("write a");
    fs::write(&file_b, &png).expect("write b");
    fs::write(&file_c, &not_image).expect("write c");

    let files = vec![file_a, file_b, file_c];

    let stats = export_images(&files, &output_dir, &ExportOptions { dedupe: true }).expect("export");

    assert_eq!(stats.scanned_files, 3);
    assert_eq!(stats.exported_files, 1);
    assert_eq!(stats.skipped_duplicate, 1);
    assert_eq!(stats.skipped_not_image, 1);

    let exported = fs::read_dir(&output_dir)
        .expect("read output")
        .filter_map(Result::ok)
        .count();
    assert_eq!(exported, 1);

    fs::remove_dir_all(base).ok();
}

#[test]
fn exports_duplicates_when_dedupe_disabled() {
    let base = unique_test_dir("keep_dupes");
    let source_dir = base.join("source");
    let output_dir = base.join("output");
    fs::create_dir_all(&source_dir).expect("create source");

    let png = vec![
        0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, b'I', b'H',
        b'D', b'R',
    ];

    let file_a = source_dir.join("a.bin");
    let file_b = source_dir.join("b.bin");
    fs::write(&file_a, &png).expect("write a");
    fs::write(&file_b, &png).expect("write b");

    let files = vec![file_a, file_b];
    let stats = export_images(&files, &output_dir, &ExportOptions { dedupe: false }).expect("export");

    assert_eq!(stats.exported_files, 2);
    assert_eq!(stats.skipped_duplicate, 0);

    fs::remove_dir_all(base).ok();
}

#[test]
fn scanner_respects_min_size() {
    let base = unique_test_dir("min_size");
    fs::create_dir_all(&base).expect("create base");

    let small = base.join("small.bin");
    let large = base.join("large.bin");
    fs::write(&small, vec![1_u8; 10]).expect("write small");
    fs::write(&large, vec![1_u8; 200]).expect("write large");

    let files = collect_cache_files(&base, &ScanOptions { min_size_bytes: 128 });
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].file_name().and_then(|v| v.to_str()), Some("large.bin"));

    fs::remove_dir_all(base).ok();
}

#[test]
fn cache_path_candidates_and_fallback_detection() {
    let local = unique_test_dir("paths");
    let profile = "Default";

    let candidates = chrome_cache_candidates(&local, profile);
    assert_eq!(candidates.len(), 2);
    let expected_primary = local
        .join("Google")
        .join("Chrome")
        .join("User Data")
        .join("Default")
        .join("Cache");
    let expected_fallback = local
        .join("Google")
        .join("Chrome")
        .join("User Data")
        .join("Default")
        .join("Network")
        .join("Cache");
    assert_eq!(candidates[0], expected_primary);
    assert_eq!(candidates[1], expected_fallback);

    fs::create_dir_all(&candidates[1]).expect("create network cache");
    let existing = existing_chrome_cache_dirs(&local, profile);
    assert_eq!(existing.len(), 1);
    assert_eq!(existing[0], candidates[1]);

    fs::remove_dir_all(local).ok();
}

#[test]
fn exporter_returns_error_when_output_path_is_file() {
    let base = unique_test_dir("output_error");
    let source_dir = base.join("source");
    fs::create_dir_all(&source_dir).expect("create source");

    let png = vec![
        0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, b'I', b'H',
        b'D', b'R',
    ];
    let file_a = source_dir.join("a.bin");
    fs::write(&file_a, &png).expect("write a");

    let invalid_output_path = base.join("output_as_file");
    fs::write(&invalid_output_path, b"not a directory").expect("create output file");

    let result = export_images(&[file_a], &invalid_output_path, &ExportOptions { dedupe: true });
    assert!(result.is_err());

    fs::remove_dir_all(base).ok();
}
