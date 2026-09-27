use cacheharvest::exporter::{export_images, ExportOptions};
use std::{fs, path::PathBuf};

struct TestDir(PathBuf);
impl TestDir {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "cacheharvest_export_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}
impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
// Complete, one-pixel GIF, including its color table, image data and trailer.
const GIF: &[u8] = b"GIF89a\x01\x00\x01\x00\x80\x00\x00\x00\x00\x00\xff\xff\xff\x2c\x00\x00\x00\x00\x01\x00\x01\x00\x00\x02\x02\x44\x01\x00\x3b";

#[test]
fn reruns_dedupe_actual_files_and_record_every_source() {
    let dir = TestDir::new();
    let source = dir.0.join("source");
    let output = dir.0.join("output");
    fs::write(&source, GIF).unwrap();
    let options = ExportOptions { dedupe: true };
    assert_eq!(
        export_images(std::slice::from_ref(&source), &output, &options)
            .unwrap()
            .exported_files,
        1
    );
    fs::rename(output.join("0001.gif"), output.join("renamed.gif")).unwrap();
    let stats = export_images(std::slice::from_ref(&source), &output, &options).unwrap();
    assert_eq!(stats.exported_files, 0);
    assert_eq!(stats.skipped_duplicate, 1);
    let manifest = fs::read_to_string(output.join("manifest.jsonl")).unwrap();
    let rows: Vec<serde_json::Value> = manifest
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[1]["duplicate"], true);
    assert!(rows[1]["output_path"]
        .as_str()
        .unwrap()
        .ends_with("renamed.gif"));
    assert_eq!(rows[0]["sha256"], rows[1]["sha256"]);
    fs::remove_file(output.join("renamed.gif")).unwrap();
    assert_eq!(
        export_images(&[source], &output, &options)
            .unwrap()
            .exported_files,
        1
    );
}

#[test]
fn disabled_dedupe_never_overwrites_existing_files() {
    let dir = TestDir::new();
    let source = dir.0.join("source");
    let output = dir.0.join("output");
    fs::write(&source, GIF).unwrap();
    fs::create_dir_all(&output).unwrap();
    fs::write(output.join("0001.gif"), b"user content").unwrap();
    let options = ExportOptions { dedupe: false };
    for _ in 0..2 {
        assert_eq!(
            export_images(std::slice::from_ref(&source), &output, &options)
                .unwrap()
                .exported_files,
            1
        );
    }
    assert_eq!(fs::read(output.join("0001.gif")).unwrap(), b"user content");
    assert_eq!(fs::read(output.join("0002.gif")).unwrap(), GIF);
    assert_eq!(fs::read(output.join("0003.gif")).unwrap(), GIF);
}

#[test]
fn manifest_failure_is_reported_before_exporting() {
    let dir = TestDir::new();
    let source = dir.0.join("source");
    let output = dir.0.join("output");
    fs::write(&source, GIF).unwrap();
    fs::create_dir_all(output.join("manifest.jsonl")).unwrap();
    assert!(export_images(&[source], &output, &ExportOptions { dedupe: false }).is_err());
    assert!(!output.join("0001.gif").exists());
}

#[test]
fn partitioned_cache_manifest_preserves_key_without_guessing_request_url() {
    let dir = TestDir::new();
    let source = dir.0.join("entry_0");
    let duplicate_source = dir.0.join("duplicate_0");
    let output = dir.0.join("output");
    let fixture = include_bytes!("fixtures/simple-v5-checks.bin");
    fs::write(&source, fixture).unwrap();
    fs::write(&duplicate_source, fixture).unwrap();
    let stats = export_images(
        &[source.clone(), duplicate_source.clone()],
        &output,
        &ExportOptions { dedupe: true },
    )
    .unwrap();
    assert_eq!(stats.exported_files, 1);
    assert_eq!(stats.skipped_duplicate, 1);
    assert_eq!(
        fs::read(output.join("0001.png")).unwrap(),
        include_bytes!("fixtures/pixel.png")
    );
    let manifest = fs::read_to_string(output.join("manifest.jsonl")).unwrap();
    let rows: Vec<serde_json::Value> = manifest
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(rows.len(), 2);
    for (index, row) in rows.iter().enumerate() {
        assert_eq!(
            row["cache_key"],
            "1/0/_dk_https://example.test https://example.test/pixel.png"
        );
        assert!(row["url"].is_null());
        assert_eq!(row["mime"], "image/png");
        assert_eq!(
            row["output_path"],
            output.join("0001.png").to_string_lossy().as_ref()
        );
        assert_eq!(row["duplicate"], index == 1);
    }
    assert_eq!(rows[0]["source_path"], source.to_string_lossy().as_ref());
    assert_eq!(
        rows[1]["source_path"],
        duplicate_source.to_string_lossy().as_ref()
    );
    assert_eq!(rows[0]["sha256"], rows[1]["sha256"]);
}
