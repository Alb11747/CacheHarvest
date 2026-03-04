use std::fs;

use cacheharvest::exporter::{export_images, ExportOptions};

#[test]
fn exports_only_images_and_skips_duplicates() {
    let base = std::env::temp_dir().join(format!(
        "cacheharvest_test_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time")
            .as_nanos()
    ));

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

    let stats = export_images(
        &files,
        &output_dir,
        &ExportOptions { dedupe: true },
    );

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
