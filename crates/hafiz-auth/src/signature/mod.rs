//! AWS Signature V4 implementation

use chrono::{DateTime, NaiveDateTime, Utc};
use hafiz_core::{Error, Result};
use hafiz_crypto::{hmac_sha256, sha256_hash};
use std::collections::BTreeMap;
use tracing::debug;

#[derive(Debug, Clone)]
pub struct SignatureV4 {
    pub access_key: String,
    pub signature: String,
    pub signed_headers: Vec<String>,
    pub date: DateTime<Utc>,
    pub region: String,
    pub service: String,
}

impl SignatureV4 {
    /// Parse Authorization header
    pub fn parse(auth_header: &str) -> Result<Self> {
        // Format: AWS4-HMAC-SHA256 Credential=.../date/region/s3/aws4_request, SignedHeaders=..., Signature=...
        if !auth_header.starts_with("AWS4-HMAC-SHA256 ") {
            return Err(Error::InvalidRequest("Invalid authorization header".into()));
        }

        let parts: Vec<&str> = auth_header[17..].split(", ").collect();

        let mut credential = None;
        let mut signed_headers = None;
        let mut signature = None;

        for part in parts {
            if let Some(val) = part.strip_prefix("Credential=") {
                credential = Some(val);
            } else if let Some(val) = part.strip_prefix("SignedHeaders=") {
                signed_headers = Some(val);
            } else if let Some(val) = part.strip_prefix("Signature=") {
                signature = Some(val);
            }
        }

        let credential =
            credential.ok_or_else(|| Error::InvalidRequest("Missing Credential".into()))?;
        let signed_headers =
            signed_headers.ok_or_else(|| Error::InvalidRequest("Missing SignedHeaders".into()))?;
        let signature =
            signature.ok_or_else(|| Error::InvalidRequest("Missing Signature".into()))?;

        // Parse credential: access_key/date/region/service/aws4_request
        let cred_parts: Vec<&str> = credential.split('/').collect();
        if cred_parts.len() != 5 {
            return Err(Error::InvalidRequest("Invalid credential format".into()));
        }

        let access_key = cred_parts[0].to_string();
        let date_str = cred_parts[1];
        let region = cred_parts[2].to_string();
        let service = cred_parts[3].to_string();

        let date =
            NaiveDateTime::parse_from_str(&format!("{}T000000Z", date_str), "%Y%m%dT%H%M%SZ")
                .map_err(|_| Error::InvalidRequest("Invalid date format".into()))?
                .and_utc();

        Ok(SignatureV4 {
            access_key,
            signature: signature.to_string(),
            signed_headers: signed_headers.split(';').map(String::from).collect(),
            date,
            region,
            service,
        })
    }
}

/// Verify AWS Signature V4
pub fn verify_signature_v4(
    method: &str,
    uri: &str,
    query_string: &str,
    headers: &BTreeMap<String, String>,
    payload_hash: &str,
    secret_key: &str,
    sig: &SignatureV4,
) -> Result<bool> {
    let amz_date = headers
        .get("x-amz-date")
        .ok_or_else(|| Error::MissingHeader("x-amz-date".into()))?;

    // Create canonical request
    let canonical_uri = uri_encode_path(uri);
    let canonical_query = canonicalize_query_string(query_string);
    let canonical_headers = canonicalize_headers(headers, &sig.signed_headers);
    let signed_headers_str = sig.signed_headers.join(";");

    let canonical_request = format!(
        "{}\n{}\n{}\n{}\n{}\n{}",
        method, canonical_uri, canonical_query, canonical_headers, signed_headers_str, payload_hash
    );

    debug!("Canonical request:\n{}", canonical_request);

    let canonical_request_hash = sha256_hash(canonical_request.as_bytes());

    // Create string to sign
    let date_stamp = &amz_date[..8];
    let credential_scope = format!("{}/{}/{}/aws4_request", date_stamp, sig.region, sig.service);

    let string_to_sign = format!(
        "AWS4-HMAC-SHA256\n{}\n{}\n{}",
        amz_date, credential_scope, canonical_request_hash
    );

    debug!("String to sign:\n{}", string_to_sign);

    // Calculate signature
    let k_date = hmac_sha256(
        format!("AWS4{}", secret_key).as_bytes(),
        date_stamp.as_bytes(),
    );
    let k_region = hmac_sha256(&k_date, sig.region.as_bytes());
    let k_service = hmac_sha256(&k_region, sig.service.as_bytes());
    let k_signing = hmac_sha256(&k_service, b"aws4_request");

    let calculated_signature = hex::encode(hmac_sha256(&k_signing, string_to_sign.as_bytes()));

    debug!("Calculated signature: {}", calculated_signature);
    debug!("Provided signature: {}", sig.signature);

    Ok(calculated_signature == sig.signature)
}

fn uri_encode_path(path: &str) -> String {
    if path.is_empty() || path == "/" {
        return "/".to_string();
    }

    path.split('/')
        .map(|segment| {
            percent_encoding::utf8_percent_encode(segment, percent_encoding::NON_ALPHANUMERIC)
                .to_string()
                .replace("%2F", "/")
                .replace("%7E", "~")
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn canonicalize_query_string(query: &str) -> String {
    if query.is_empty() {
        return String::new();
    }

    let mut params: Vec<(String, String)> = query
        .split('&')
        .filter(|p| !p.is_empty())
        .map(|p| {
            let mut parts = p.splitn(2, '=');
            let key = parts.next().unwrap_or("");
            let value = parts.next().unwrap_or("");
            (key.to_string(), value.to_string())
        })
        .collect();

    params.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));

    params
        .into_iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("&")
}

fn canonicalize_headers(headers: &BTreeMap<String, String>, signed_headers: &[String]) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_signature() {
        let header = "AWS4-HMAC-SHA256 Credential=AKIAIOSFODNN7EXAMPLE/20130524/us-east-1/s3/aws4_request, SignedHeaders=host;range;x-amz-date, Signature=fe5f80f77d5fa3beca038a248ff027d0445342fe2855ddc963176630326f1024";

        let sig = SignatureV4::parse(header).unwrap();

        assert_eq!(sig.access_key, "AKIAIOSFODNN7EXAMPLE");
        assert_eq!(sig.region, "us-east-1");
        assert_eq!(sig.service, "s3");
        assert_eq!(sig.signed_headers, vec!["host", "range", "x-amz-date"]);
    }
}
