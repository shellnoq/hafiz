//! TLS/HTTPS support for Hafiz
//!
//! Provides secure HTTPS connections with support for:
//! - TLS 1.2 and 1.3
//! - mTLS (mutual TLS) for client certificate verification
//! - HSTS headers
//! - Self-signed certificate generation for development

use hafiz_core::config::{TlsConfig, TlsVersion};
use hafiz_core::{Error, Result};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::Arc;
use tokio_rustls::rustls::{
    self,
    pki_types::{CertificateDer, PrivateKeyDer},
    server::WebPkiClientVerifier,
    RootCertStore,
};
use tracing::{info, warn};

/// TLS Acceptor wrapper for async TLS connections
pub struct TlsAcceptor {
    acceptor: tokio_rustls::TlsAcceptor,
    hsts_enabled: bool,
    hsts_max_age: u64,
}

impl TlsAcceptor {
    /// Create a new TLS acceptor from configuration
    pub fn from_config(config: &TlsConfig) -> Result<Self> {
        config.validate()?;

        let cert_file = config.cert_file.as_ref().ok_or_else(|| {
            Error::InvalidArgument("Certificate file not specified".into())
        })?;
        let key_file = config.key_file.as_ref().ok_or_else(|| {
            Error::InvalidArgument("Key file not specified".into())
        })?;

        // Load certificates
        let certs = load_certs(cert_file)?;
        info!("Loaded {} certificate(s)", certs.len());

        // Load private key
        let key = load_private_key(key_file)?;
        info!("Loaded private key");

        // Build server config
        let mut server_config = if config.require_client_cert {
            // mTLS: require client certificates
            let client_ca_file = config.client_ca_file.as_ref().ok_or_else(|| {
                Error::InvalidArgument("Client CA file required for mTLS".into())
            })?;
            
            let client_roots = load_root_certs(client_ca_file)?;
            info!("Loaded {} client CA certificate(s)", client_roots.len());
            
            let mut root_store = RootCertStore::empty();
            for cert in client_roots {
                root_store.add(cert).map_err(|e| {
                    Error::InternalError(format!("Failed to add client CA cert: {}", e))
                })?;
            }
            
            let client_verifier = WebPkiClientVerifier::builder(Arc::new(root_store))
                .build()
                .map_err(|e| {
                    Error::InternalError(format!("Failed to build client verifier: {}", e))
                })?;
            
            rustls::ServerConfig::builder()
                .with_client_cert_verifier(client_verifier)
                .with_single_cert(certs, key)
                .map_err(|e| Error::InternalError(format!("TLS config error: {}", e)))?
        } else {
            // Standard TLS: no client certificates
            rustls::ServerConfig::builder()
                .with_no_client_auth()
                .with_single_cert(certs, key)
                .map_err(|e| Error::InternalError(format!("TLS config error: {}", e)))?
        };

        // Set minimum TLS version
        let min_version = match config.min_version {
            TlsVersion::Tls12 => &rustls::version::TLS12,
            TlsVersion::Tls13 => &rustls::version::TLS13,
        };
        server_config.versions = vec![min_version.clone(), rustls::version::TLS13.clone()];
        
        // Enable ALPN for HTTP/1.1 and HTTP/2
        server_config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

        let acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(server_config));

        Ok(Self {
            acceptor,
            hsts_enabled: config.hsts_enabled,
            hsts_max_age: config.hsts_max_age,
        })
    }

    /// Get the inner TLS acceptor
    pub fn inner(&self) -> &tokio_rustls::TlsAcceptor {
        &self.acceptor
    }

    /// Check if HSTS is enabled
    pub fn hsts_enabled(&self) -> bool {
        self.hsts_enabled
    }

    /// Get HSTS header value
    pub fn hsts_header(&self) -> Option<String> {
        if self.hsts_enabled {
            Some(format!(
                "max-age={}; includeSubDomains; preload",
                self.hsts_max_age
            ))
        } else {
            None
        }
    }
}

/// Load certificates from PEM file
fn load_certs(path: &Path) -> Result<Vec<CertificateDer<'static>>> {
    let file = File::open(path).map_err(|e| {
        Error::InternalError(format!("Failed to open certificate file {:?}: {}", path, e))
    })?;
    let mut reader = BufReader::new(file);
    
    let certs: Vec<CertificateDer<'static>> = rustls_pemfile::certs(&mut reader)
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| Error::InternalError(format!("Failed to parse certificates: {}", e)))?;
    
    if certs.is_empty() {
        return Err(Error::InvalidArgument(format!(
            "No certificates found in {:?}",
            path
        )));
    }
    
    Ok(certs)
}

/// Load private key from PEM file
fn load_private_key(path: &Path) -> Result<PrivateKeyDer<'static>> {
    let file = File::open(path).map_err(|e| {
        Error::InternalError(format!("Failed to open key file {:?}: {}", path, e))
    })?;
    let mut reader = BufReader::new(file);
    
    // Try different key formats
    loop {
        match rustls_pemfile::read_one(&mut reader) {
            Ok(Some(rustls_pemfile::Item::Pkcs1Key(key))) => {
                return Ok(PrivateKeyDer::Pkcs1(key));
            }
            Ok(Some(rustls_pemfile::Item::Pkcs8Key(key))) => {
                return Ok(PrivateKeyDer::Pkcs8(key));
            }
            Ok(Some(rustls_pemfile::Item::Sec1Key(key))) => {
                return Ok(PrivateKeyDer::Sec1(key));
            }
            Ok(None) => break,
            Ok(Some(_)) => continue, // Skip other items like certs
            Err(e) => {
                return Err(Error::InternalError(format!(
                    "Failed to parse private key: {}",
                    e
                )));
            }
        }
    }
    
    Err(Error::InvalidArgument(format!(
        "No private key found in {:?}",
        path
    )))
}

/// Load root certificates from PEM file
fn load_root_certs(path: &Path) -> Result<Vec<CertificateDer<'static>>> {
    load_certs(path)
}

/// Generate self-signed certificate for development
/// 
/// This generates a certificate valid for localhost and 127.0.0.1
pub fn generate_self_signed_cert(
    output_cert: &Path,
    output_key: &Path,
    days_valid: u32,
) -> Result<()> {
    use rcgen::{Certificate, CertificateParams, DistinguishedName, DnType, SanType};
    
    let mut params = CertificateParams::default();
    
    // Set subject
    let mut dn = DistinguishedName::new();
    dn.push(DnType::CommonName, "Hafiz Development");
    dn.push(DnType::OrganizationName, "Hafiz");
    params.distinguished_name = dn;
    
    // Set validity
    params.not_before = time::OffsetDateTime::now_utc();
    params.not_after = params.not_before + time::Duration::days(days_valid as i64);
    
    // Set SANs (Subject Alternative Names)
    params.subject_alt_names = vec![
        SanType::DnsName("localhost".try_into().unwrap()),
        SanType::DnsName("*.localhost".try_into().unwrap()),
        SanType::IpAddress(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1))),
        SanType::IpAddress(std::net::IpAddr::V6(std::net::Ipv6Addr::LOCALHOST)),
    ];
    
    // Generate certificate
    let cert = Certificate::from_params(params).map_err(|e| {
        Error::InternalError(format!("Failed to generate certificate: {}", e))
    })?;
    
    // Write certificate PEM
    let cert_pem = cert.serialize_pem().map_err(|e| {
        Error::InternalError(format!("Failed to serialize certificate: {}", e))
    })?;
    std::fs::write(output_cert, &cert_pem).map_err(|e| {
        Error::InternalError(format!("Failed to write certificate file: {}", e))
    })?;
    
    // Write private key PEM
    let key_pem = cert.serialize_private_key_pem();
    std::fs::write(output_key, &key_pem).map_err(|e| {
        Error::InternalError(format!("Failed to write key file: {}", e))
    })?;
    
    // Set permissions on key file (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(output_key)
            .map_err(|e| Error::InternalError(format!("Failed to get key metadata: {}", e)))?
            .permissions();
        perms.set_mode(0o600);
        std::fs::set_permissions(output_key, perms)
            .map_err(|e| Error::InternalError(format!("Failed to set key permissions: {}", e)))?;
    }
    
    info!(
        "Generated self-signed certificate:\n  Certificate: {:?}\n  Private Key: {:?}\n  Valid for: {} days",
        output_cert, output_key, days_valid
    );
    
    warn!("⚠️  Self-signed certificates are for development only!");
    warn!("⚠️  Use proper certificates from a CA for production.");
    
    Ok(())
}

/// Certificate info for display
#[derive(Debug)]
pub struct CertInfo {
    pub subject: String,
    pub issuer: String,
    pub not_before: String,
    pub not_after: String,
    pub san: Vec<String>,
}

/// Get certificate information
pub fn get_cert_info(cert_path: &Path) -> Result<CertInfo> {
    use x509_parser::prelude::*;
    
    let pem_data = std::fs::read(cert_path).map_err(|e| {
        Error::InternalError(format!("Failed to read certificate: {}", e))
    })?;
    
    let (_, pem) = parse_x509_pem(&pem_data).map_err(|e| {
        Error::InternalError(format!("Failed to parse PEM: {:?}", e))
    })?;
    
    let (_, cert) = X509Certificate::from_der(&pem.contents).map_err(|e| {
        Error::InternalError(format!("Failed to parse certificate: {:?}", e))
    })?;
    
    let subject = cert.subject().to_string();
    let issuer = cert.issuer().to_string();
    let not_before = cert.validity().not_before.to_rfc2822();
    let not_after = cert.validity().not_after.to_rfc2822();
    
    let mut san = Vec::new();
    if let Ok(Some(ext)) = cert.subject_alternative_name() {
        for name in ext.value.general_names.iter() {
            san.push(format!("{:?}", name));
        }
    }
    
    Ok(CertInfo {
        subject,
        issuer,
        not_before,
        not_after,
        san,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_generate_self_signed() {
        let dir = tempdir().unwrap();
        let cert_path = dir.path().join("cert.pem");
        let key_path = dir.path().join("key.pem");
        
        generate_self_signed_cert(&cert_path, &key_path, 365).unwrap();
        
        assert!(cert_path.exists());
        assert!(key_path.exists());
        
        // Verify we can load the generated files
        let certs = load_certs(&cert_path).unwrap();
        assert_eq!(certs.len(), 1);
        
        let _key = load_private_key(&key_path).unwrap();
    }
    
    #[test]
    fn test_get_cert_info() {
        let dir = tempdir().unwrap();
        let cert_path = dir.path().join("cert.pem");
        let key_path = dir.path().join("key.pem");
        
        generate_self_signed_cert(&cert_path, &key_path, 365).unwrap();
        
        let info = get_cert_info(&cert_path).unwrap();
        assert!(info.subject.contains("Hafiz"));
    }
}
