//! Pre-signed URL generation and verification
//!
//! Implements AWS S3-compatible pre-signed URL functionality.

use chrono::{DateTime, Duration, Utc};
use hafiz_core::types::{PresignedMethod, PresignedRequest, PresignedUrl};
use hafiz_core::{Error, Result};
use hafiz_crypto::{hmac_sha256, sha256_hash};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use std::collections::BTreeMap;
use tracing::debug;
use url::Url;

/// AWS S3 presigned URL query parameters
const X_AMZ_ALGORITHM: &str = "X-Amz-Algorithm";
const X_AMZ_CREDENTIAL: &str = "X-Amz-Credential";
const X_AMZ_DATE: &str = "X-Amz-Date";
const X_AMZ_EXPIRES: &str = "X-Amz-Expires";
const X_AMZ_SIGNED_HEADERS: &str = "X-Amz-SignedHeaders";
const X_AMZ_SIGNATURE: &str = "X-Amz-Signature";
const X_AMZ_SECURITY_TOKEN: &str = "X-Amz-Security-Token";

/// Unsigned payload constant for presigned URLs
const UNSIGNED_PAYLOAD: &str = "UNSIGNED-PAYLOAD";

/// Generate a pre-signed URL for S3 operations
pub fn generate_presigned_url(
    request: &PresignedRequest,
    endpoint: &str,
    access_key: &str,
    secret_key: &str,
    region: &str,
) -> Result<PresignedUrl> {
    let now = Utc::now();
    let expires_at = now + Duration::seconds(request.expires_in as i64);

    // Format date for signing
    let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();
    let date_stamp = now.format("%Y%m%d").to_string();

    // Build the credential scope
    let credential_scope = format!("{}/{}/s3/aws4_request", date_stamp, region);
    let credential = format!("{}/{}", access_key, credential_scope);

    // Build canonical URI
    let canonical_uri = format!("/{}/{}",
        uri_encode(&request.bucket, false),
        uri_encode(&request.key, false)
    );

    // Build query string
    let mut query_params: BTreeMap<String, String> = BTreeMap::new();
    query_params.insert(X_AMZ_ALGORITHM.to_string(), "AWS4-HMAC-SHA256".to_string());
    query_params.insert(X_AMZ_CREDENTIAL.to_string(), credential.clone());
    query_params.insert(X_AMZ_DATE.to_string(), amz_date.clone());
    query_params.insert(X_AMZ_EXPIRES.to_string(), request.expires_in.to_string());
    query_params.insert(X_AMZ_SIGNED_HEADERS.to_string(), "host".to_string());

    if let Some(version_id) = &request.version_id {
        query_params.insert("versionId".to_string(), version_id.clone());
    }

    // Build canonical query string (sorted and URL encoded)
    let canonical_query_string = build_canonical_query_string(&query_params);

    // Build canonical headers
    let host = extract_host(endpoint)?;
    let canonical_headers = format!("host:{}\n", host);
    let signed_headers = "host";

    // Create canonical request
    let canonical_request = format!(
        "{}\n{}\n{}\n{}\n{}\n{}",
        request.method,
        canonical_uri,
        canonical_query_string,
        canonical_headers,
        signed_headers,
        UNSIGNED_PAYLOAD
    );

    debug!("Canonical request for presigning:\n{}", canonical_request);

    // Create string to sign
    let canonical_request_hash = sha256_hash(canonical_request.as_bytes());
    let string_to_sign = format!(
        "AWS4-HMAC-SHA256\n{}\n{}\n{}",
        amz_date, credential_scope, canonical_request_hash
    );

    debug!("String to sign:\n{}", string_to_sign);

    // Calculate signature
    let signature = calculate_signature(secret_key, &date_stamp, region, &string_to_sign);

    // Build final URL
    let mut final_url = format!("{}{}", endpoint.trim_end_matches('/'), canonical_uri);
    final_url.push('?');
    final_url.push_str(&canonical_query_string);
    final_url.push_str(&format!("&{}={}", X_AMZ_SIGNATURE, signature));

    // Prepare headers for PUT requests
    let headers = if request.method == PresignedMethod::Put {
        let mut h = Vec::new();
        if let Some(ct) = &request.content_type {
            h.push(("Content-Type".to_string(), ct.clone()));
        }
        if let Some(md5) = &request.content_md5 {
            h.push(("Content-MD5".to_string(), md5.clone()));
        }
        if h.is_empty() { None } else { Some(h) }
    } else {
        None
    };

    Ok(PresignedUrl {
        url: final_url,
        method: request.method.to_string(),
        expires_at,
        headers,
    })
}

/// Verify a pre-signed URL
pub fn verify_presigned_url(
    method: &str,
    uri: &str,
    query_string: &str,
    headers: &BTreeMap<String, String>,
    secret_key: &str,
    region: &str,
) -> Result<bool> {
    // Parse query parameters
    let params = parse_query_string(query_string);

    // Extract required parameters
    let algorithm = params.get(X_AMZ_ALGORITHM)
        .ok_or_else(|| Error::InvalidRequest("Missing X-Amz-Algorithm".into()))?;
    let credential = params.get(X_AMZ_CREDENTIAL)
        .ok_or_else(|| Error::InvalidRequest("Missing X-Amz-Credential".into()))?;
    let amz_date = params.get(X_AMZ_DATE)
        .ok_or_else(|| Error::InvalidRequest("Missing X-Amz-Date".into()))?;
    let expires = params.get(X_AMZ_EXPIRES)
        .ok_or_else(|| Error::InvalidRequest("Missing X-Amz-Expires".into()))?;
    let signed_headers = params.get(X_AMZ_SIGNED_HEADERS)
        .ok_or_else(|| Error::InvalidRequest("Missing X-Amz-SignedHeaders".into()))?;
    let provided_signature = params.get(X_AMZ_SIGNATURE)
        .ok_or_else(|| Error::InvalidRequest("Missing X-Amz-Signature".into()))?;

    // Verify algorithm
    if algorithm != "AWS4-HMAC-SHA256" {
        return Err(Error::InvalidRequest("Unsupported algorithm".into()));
    }

    // Parse and verify expiration
    let request_time = parse_amz_date(amz_date)?;
    let expires_secs: u64 = expires.parse()
        .map_err(|_| Error::InvalidRequest("Invalid expires value".into()))?;
    let expiration_time = request_time + Duration::seconds(expires_secs as i64);

    if Utc::now() > expiration_time {
        return Err(Error::ExpiredPresignedRequest);
    }

    // Extract date stamp from credential
    let cred_parts: Vec<&str> = credential.split('/').collect();
    if cred_parts.len() != 5 {
        return Err(Error::InvalidRequest("Invalid credential format".into()));
    }
    let date_stamp = cred_parts[1];
    let cred_region = cred_parts[2];

    // Verify region matches
    if cred_region != region {
        return Err(Error::InvalidRequest("Region mismatch".into()));
    }

    // Build canonical query string without signature
    let mut query_params: BTreeMap<String, String> = params.clone();
    query_params.remove(X_AMZ_SIGNATURE);
    let canonical_query_string = build_canonical_query_string(&query_params);

    // Build canonical headers
    let signed_header_list: Vec<&str> = signed_headers.split(';').collect();
    let canonical_headers = build_canonical_headers(headers, &signed_header_list);

    // Create canonical request
    let canonical_request = format!(
        "{}\n{}\n{}\n{}\n{}\n{}",
        method,
        uri,
        canonical_query_string,
        canonical_headers,
        signed_headers,
        UNSIGNED_PAYLOAD
    );

    debug!("Canonical request for verification:\n{}", canonical_request);

    // Create string to sign
    let canonical_request_hash = sha256_hash(canonical_request.as_bytes());
    let credential_scope = format!("{}/{}/s3/aws4_request", date_stamp, region);
    let string_to_sign = format!(
        "AWS4-HMAC-SHA256\n{}\n{}\n{}",
        amz_date, credential_scope, canonical_request_hash
    );

    // Calculate expected signature
    let expected_signature = calculate_signature(secret_key, date_stamp, region, &string_to_sign);

    debug!("Expected signature: {}", expected_signature);
    debug!("Provided signature: {}", provided_signature);

    Ok(expected_signature == *provided_signature)
}

/// Extract access key from pre-signed URL query parameters
pub fn extract_access_key_from_presigned(query_string: &str) -> Result<String> {
    let params = parse_query_string(query_string);

    let credential = params.get(X_AMZ_CREDENTIAL)
        .ok_or_else(|| Error::InvalidRequest("Missing X-Amz-Credential".into()))?;

    let cred_parts: Vec<&str> = credential.split('/').collect();
    if cred_parts.is_empty() {
        return Err(Error::InvalidRequest("Invalid credential format".into()));
    }

    Ok(cred_parts[0].to_string())
}

/// Check if a request is a pre-signed URL request
pub fn is_presigned_request(query_string: &str) -> bool {
    let params = parse_query_string(query_string);
    params.contains_key(X_AMZ_ALGORITHM) && params.contains_key(X_AMZ_SIGNATURE)
}

// Helper functions

fn calculate_signature(secret_key: &str, date_stamp: &str, region: &str, string_to_sign: &str) -> String {
    let k_date = hmac_sha256(format!("AWS4{}", secret_key).as_bytes(), date_stamp.as_bytes());
    let k_region = hmac_sha256(&k_date, region.as_bytes());
    let k_service = hmac_sha256(&k_region, b"s3");
    let k_signing = hmac_sha256(&k_service, b"aws4_request");
    hex::encode(hmac_sha256(&k_signing, string_to_sign.as_bytes()))
}

fn uri_encode(input: &str, encode_slash: bool) -> String {
    let mut result = String::new();
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '.' || ch == '~' {
            result.push(ch);
        } else if ch == '/' && !encode_slash {
            result.push(ch);
        } else {
            for byte in ch.to_string().as_bytes() {
                result.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    result
}

fn build_canonical_query_string(params: &BTreeMap<String, String>) -> String {
    params
        .iter()
        .map(|(k, v)| format!("{}={}", uri_encode(k, true), uri_encode(v, true)))
        .collect::<Vec<_>>()
        .join("&")
}

fn build_canonical_headers(headers: &BTreeMap<String, String>, signed_headers: &[&str]) -> String {
    let mut result = String::new();
    for header in signed_headers {
        let header_lower = header.to_lowercase();
        if let Some(value) = headers.get(&header_lower) {
            result.push_str(&header_lower);
            result.push(':');
            result.push_str(value.trim());
            result.push('\n');
        }
    }
    result
}

fn parse_query_string(query: &str) -> BTreeMap<String, String> {
    let mut params = BTreeMap::new();
    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }
        let mut parts = pair.splitn(2, '=');
        let key = parts.next().unwrap_or("");
        let value = parts.next().unwrap_or("");
        // URL decode
        let key = urlencoding::decode(key).unwrap_or_else(|_| key.into()).to_string();
        let value = urlencoding::decode(value).unwrap_or_else(|_| value.into()).to_string();
        params.insert(key, value);
    }
    params
}

fn parse_amz_date(date_str: &str) -> Result<DateTime<Utc>> {
    chrono::NaiveDateTime::parse_from_str(date_str, "%Y%m%dT%H%M%SZ")
        .map(|dt| dt.and_utc())
        .map_err(|_| Error::InvalidRequest("Invalid X-Amz-Date format".into()))
}

fn extract_host(endpoint: &str) -> Result<String> {
    Url::parse(endpoint)
        .map_err(|_| Error::InvalidRequest("Invalid endpoint URL".into()))?
        .host_str()
        .map(|h| {
            let port = Url::parse(endpoint).unwrap().port();
            if let Some(p) = port {
                format!("{}:{}", h, p)
            } else {
                h.to_string()
            }
        })
        .ok_or_else(|| Error::InvalidRequest("No host in endpoint".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_presigned_url() {
        let request = PresignedRequest {
            method: PresignedMethod::Get,
            bucket: "my-bucket".to_string(),
            key: "my-object.txt".to_string(),
            expires_in: 3600,
            ..Default::default()
        };

        let result = generate_presigned_url(
            &request,
            "http://localhost:9000",
            "minioadmin",
            "minioadmin",
            "us-east-1",
        );

        assert!(result.is_ok());
        let presigned = result.unwrap();
        assert!(presigned.url.contains("X-Amz-Algorithm=AWS4-HMAC-SHA256"));
        assert!(presigned.url.contains("X-Amz-Credential="));
        assert!(presigned.url.contains("X-Amz-Signature="));
        assert_eq!(presigned.method, "GET");
    }

    #[test]
    fn test_is_presigned_request() {
        assert!(is_presigned_request("X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Signature=abc"));
        assert!(!is_presigned_request("foo=bar"));
        assert!(!is_presigned_request(""));
    }

    #[test]
    fn test_extract_access_key() {
        let query = "X-Amz-Credential=AKIAIOSFODNN7EXAMPLE%2F20130524%2Fus-east-1%2Fs3%2Faws4_request";
        let result = extract_access_key_from_presigned(query);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "AKIAIOSFODNN7EXAMPLE");
    }
}
