//! Hash utilities

use base64::{engine::general_purpose::STANDARD, Engine};
use digest::Digest;
use hmac::{Hmac, Mac};
use md5::Md5;
use sha1::Sha1;
use sha2::Sha256;

pub fn md5_hash(data: &[u8]) -> String {
    let mut hasher = Md5::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

pub fn sha256_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

pub fn sha1_hash(data: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

pub fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC can take key of any size");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

pub fn hmac_sha256_hex(key: &[u8], data: &[u8]) -> String {
    hex::encode(hmac_sha256(key, data))
}

pub fn md5_base64(data: &[u8]) -> String {
    let mut hasher = Md5::new();
    hasher.update(data);
    STANDARD.encode(hasher.finalize())
}

/// Calculate multipart upload ETag
/// Format: MD5(concat(part_md5s))-part_count
pub fn multipart_etag(part_etags: &[String], part_count: usize) -> String {
    let mut hasher = Md5::new();

    for etag in part_etags {
        // Remove quotes and decode hex
        let clean = etag.trim_matches('"');
        if let Ok(bytes) = hex::decode(clean) {
            hasher.update(&bytes);
        }
    }

    let hash = hasher.finalize();
    format!("{}-{}", hex::encode(hash), part_count)
}
