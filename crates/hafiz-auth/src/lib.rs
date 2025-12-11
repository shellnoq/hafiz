//! Authentication for Hafiz

pub mod ldap;
pub mod presigned;
pub mod signature;

pub use ldap::{
    LdapAuthProvider, LdapClient, LdapConfig, LdapUser, LdapAuthResult,
    LdapStatus, LdapServerType, AttributeMappings,
};
pub use presigned::{
    generate_presigned_url, verify_presigned_url,
    extract_access_key_from_presigned, is_presigned_request,
};
pub use signature::{SignatureV4, verify_signature_v4};

use rand::Rng;

/// Generate new access key and secret key pair
pub fn generate_credentials() -> (String, String) {
    let mut rng = rand::thread_rng();

    // Access key: AKIA + 16 alphanumeric chars (like AWS)
    let access_key: String = format!(
        "AKIA{}",
        (0..16)
            .map(|_| {
                let idx = rng.gen_range(0..36);
                if idx < 10 {
                    (b'0' + idx) as char
                } else {
                    (b'A' + idx - 10) as char
                }
            })
            .collect::<String>()
    );

    // Secret key: 40 characters base64-like
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let secret_key: String = (0..40)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

    (access_key, secret_key)
}
