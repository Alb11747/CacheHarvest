use std::{fs, process::Command};

#[test]
fn cli_recovers_cache_body_and_dedupes_on_second_run() {
    let base = std::env::temp_dir().join(format!("cacheharvest_cli_{}", std::process::id()));
    let source = base.join("source");
    let output = base.join("output");
    fs::create_dir_all(&source).unwrap();
    fs::write(
        source.join("entry_0"),
        include_bytes!("fixtures/simple-v5-checks.bin"),
    )
    .unwrap();
    for expected in ["Exported        : 1", "Exported        : 0"] {
        let result = Command::new(env!("CARGO_BIN_EXE_cacheharvest"))
            .arg(&output)
            .arg("--cache-dir")
            .arg(&source)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(String::from_utf8_lossy(&result.stdout).contains(expected));
    }
    assert_eq!(
        fs::read(output.join("0001.png")).unwrap(),
        include_bytes!("fixtures/pixel.png")
    );
    fs::write(
        source.join("entry_0"),
        &include_bytes!("fixtures/simple-v5-checks.bin")[..128],
    )
    .unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_cacheharvest"))
        .arg(&output)
        .arg("--cache-dir")
        .arg(&source)
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert_eq!(
        fs::read(output.join("0001.png")).unwrap(),
        include_bytes!("fixtures/pixel.png")
    );
    fs::remove_dir_all(base).unwrap();
}
