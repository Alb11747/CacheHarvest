use cacheharvest::cache_entry::extract;

const PNG: &[u8] = include_bytes!("fixtures/pixel.png");
const CHECKED: &[u8] = include_bytes!("fixtures/simple-v5-checks.bin");
const UNCHECKED: &[u8] = include_bytes!("fixtures/simple-v5-unchecked.bin");
const KEY: &str = "1/0/_dk_https://example.test https://example.test/pixel.png";
const BODY_START: usize = 24 + KEY.len();
const BODY_EOF: usize = BODY_START + PNG.len();

#[test]
fn extracts_exact_body_and_key_with_and_without_optional_checks() {
    for bytes in [CHECKED, UNCHECKED] {
        let payload = extract(bytes).expect("supported cache entry");
        assert_eq!(payload.data, PNG);
        assert_eq!(payload.key.as_deref(), Some(KEY));
    }
}

#[test]
fn raw_png_is_preserved_without_a_cache_key() {
    let payload = extract(PNG).expect("raw image");
    assert_eq!(payload.data, PNG);
    assert!(payload.key.is_none());
}

#[test]
fn rejects_every_truncation_after_the_cache_magic() {
    for length in 8..CHECKED.len() {
        assert!(
            extract(&CHECKED[..length]).is_err(),
            "accepted length {length}"
        );
    }
}

#[test]
fn rejects_unsupported_version_and_impossible_lengths() {
    for (offset, value) in [(8, 6u32), (12, u32::MAX), (CHECKED.len() - 8, u32::MAX)] {
        let mut bytes = CHECKED.to_vec();
        bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        assert!(
            extract(&bytes).is_err(),
            "accepted invalid field at {offset}"
        );
    }
}

#[test]
fn rejects_corrupted_body_metadata_digest_and_footers() {
    for offset in [
        BODY_START + 40,
        BODY_EOF + 24,
        CHECKED.len() - 24 - 32,
        BODY_EOF,
        CHECKED.len() - 24,
        24,
    ] {
        let mut bytes = CHECKED.to_vec();
        bytes[offset] ^= 0x40;
        assert!(extract(&bytes).is_err(), "accepted corruption at {offset}");
    }
}

#[test]
fn crc_fields_are_only_checked_when_their_flag_is_set() {
    let mut bytes = UNCHECKED.to_vec();
    for offset in [BODY_EOF + 12, UNCHECKED.len() - 12] {
        bytes[offset..offset + 4].copy_from_slice(&0x12345678u32.to_le_bytes());
    }
    assert_eq!(extract(&bytes).expect("CRC flag is absent").data, PNG);
    for offset in [BODY_EOF + 8, UNCHECKED.len() - 16] {
        let mut flagged = bytes.clone();
        flagged[offset] |= 1;
        assert!(extract(&flagged).is_err(), "ignored CRC flag at {offset}");
    }
}

#[test]
fn rejects_unknown_footer_flags_and_misplaced_digest_flag() {
    for (offset, flag) in [
        (BODY_EOF + 8, 4),
        (CHECKED.len() - 16, 4),
        (BODY_EOF + 8, 2),
    ] {
        let mut bytes = CHECKED.to_vec();
        bytes[offset] |= flag;
        assert!(extract(&bytes).is_err());
    }
}

#[test]
fn rejects_stream_two_and_sparse_entries() {
    // A stream-2 file has one body and footer, with no stream-0 metadata/footer.
    let stream_two = &UNCHECKED[..BODY_EOF + 24];
    assert!(extract(stream_two).is_err());
    let sparse = 0xeb97bf016553676bu64.to_le_bytes();
    assert!(extract(&sparse).is_err());
}
