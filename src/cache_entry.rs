//! Chromium Simple Cache v5 combined stream 0/1 reader.
//! Layout: https://chromium.googlesource.com/chromium/src/+/main/net/disk_cache/simple/simple_entry_format.h
use sha2::{Digest, Sha256};

const INITIAL: u64 = 0xfcfb6d1ba7725c30;
const FINAL: u64 = 0xf4fa6f45970d41d8;
const SPARSE: u64 = 0xeb97bf016553676b;
const RECORD: usize = 24;

pub struct CachePayload<'a> {
    pub data: &'a [u8],
    pub key: Option<String>,
}

fn u32_at(bytes: &[u8], offset: usize) -> Result<u32, String> {
    let data = bytes
        .get(offset..offset + 4)
        .ok_or("Truncated cache record")?;
    Ok(u32::from_le_bytes(data.try_into().unwrap()))
}
fn magic(bytes: &[u8]) -> Option<u64> {
    Some(u64::from_le_bytes(bytes.get(..8)?.try_into().ok()?))
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = !0u32;
    for byte in bytes {
        crc ^= *byte as u32;
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xedb88320 & 0u32.wrapping_sub(crc & 1));
        }
    }
    !crc
}

fn footer(bytes: &[u8]) -> Result<(u32, u32, usize), String> {
    if bytes.len() != RECORD || magic(bytes) != Some(FINAL) {
        return Err("Invalid or missing cache footer".into());
    }
    let flags = u32_at(bytes, 8)?;
    if flags & !3 != 0 {
        return Err("Unsupported cache footer flags".into());
    }
    Ok((flags, u32_at(bytes, 12)?, u32_at(bytes, 16)? as usize))
}

/// Return a raw file or the isolated response body from a supported cache entry.
/// Never carve signatures: that could export metadata or a truncated response.
pub fn extract(bytes: &[u8]) -> Result<CachePayload<'_>, String> {
    if magic(bytes) == Some(SPARSE) {
        return Err("Sparse cache entries are unsupported".into());
    }
    if magic(bytes) != Some(INITIAL) {
        return Ok(CachePayload {
            data: bytes,
            key: None,
        });
    }
    if bytes.len() < RECORD * 3 || u32_at(bytes, 8)? != 5 {
        return Err("Truncated or unsupported Simple Cache entry (requires v5)".into());
    }
    let key_end = RECORD
        .checked_add(u32_at(bytes, 12)? as usize)
        .ok_or("Invalid key length")?;
    let key = bytes.get(RECORD..key_end).ok_or("Truncated cache key")?;
    let end = bytes.len() - RECORD;
    let (flags0, crc0, size0) = footer(&bytes[end..])?;
    let metadata_end = end
        .checked_sub(if flags0 & 2 != 0 { 32 } else { 0 })
        .ok_or("Missing key digest")?;
    if flags0 & 2 != 0 && Sha256::digest(key).as_slice() != &bytes[metadata_end..end] {
        return Err("Cache key digest mismatch".into());
    }
    let metadata_start = metadata_end
        .checked_sub(size0)
        .ok_or("Invalid metadata length")?;
    let body_end = metadata_start
        .checked_sub(RECORD)
        .ok_or("Missing response footer")?;
    let body = bytes
        .get(key_end..body_end)
        .ok_or("Invalid response bounds")?;
    let (flags1, crc1, _) = footer(&bytes[body_end..metadata_start])?;
    if flags1 & 2 != 0 {
        return Err("Unexpected key digest flag on response stream".into());
    }
    if (flags0 & 1 != 0 && crc32(&bytes[metadata_start..metadata_end]) != crc0)
        || (flags1 & 1 != 0 && crc32(body) != crc1)
    {
        return Err("Cache stream checksum mismatch".into());
    }
    let key = String::from_utf8(key.to_vec()).map_err(|_| "Cache key is not UTF-8")?;
    Ok(CachePayload {
        data: body,
        key: Some(key),
    })
}
