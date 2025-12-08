//! Server-Side Encryption (SSE) Implementation
//! 
//! Supports:
//! - SSE-S3: Server-managed keys (AES-256-GCM)
//! - SSE-C: Customer-provided keys
//! 
//! Architecture:
//! - Master Encryption Key (MEK): Stored securely, used to encrypt DEKs
//! - Data Encryption Key (DEK): Per-object random key, encrypted with MEK
//! - Envelope encryption: DEK encrypts data, MEK encrypts DEK

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use rand::RngCore;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use thiserror::Error;

/// Encryption errors
#[derive(Debug, Error)]
pub enum EncryptionError {
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),
    
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),
    
    #[error("Invalid key: {0}")]
    InvalidKey(String),
    
    #[error("Key derivation failed: {0}")]
    KeyDerivationFailed(String),
}

/// Server-Side Encryption type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SseType {
    /// No encryption
    None,
    /// SSE-S3: Server-managed keys
    SseS3,
    /// SSE-C: Customer-provided keys
    SseC,
}

impl SseType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "",
            Self::SseS3 => "AES256",
            Self::SseC => "AES256",
        }
    }
    
    pub fn from_header(header: Option<&str>) -> Self {
        match header {
            Some("AES256") => Self::SseS3,
            Some("aws:kms") => Self::SseS3, // Treat KMS as S3 for now
            _ => Self::None,
        }
    }
}

/// Encrypted object metadata
#[derive(Debug, Clone)]
pub struct EncryptedObjectInfo {
    /// Encryption type used
    pub sse_type: SseType,
    /// Encrypted Data Encryption Key (for SSE-S3)
    pub encrypted_dek: Option<Vec<u8>>,
    /// Nonce/IV used for DEK encryption
    pub dek_nonce: Option<Vec<u8>>,
    /// Nonce/IV used for data encryption
    pub data_nonce: Vec<u8>,
    /// MD5 of customer key (for SSE-C)
    pub sse_customer_key_md5: Option<String>,
}

/// Key Manager for SSE-S3
pub struct KeyManager {
    /// Master Encryption Key (256-bit)
    master_key: [u8; 32],
    /// Cipher for MEK operations
    mek_cipher: Aes256Gcm,
}

impl KeyManager {
    /// Create new KeyManager with master key from config/env
    pub fn new(master_key: &[u8]) -> Result<Self, EncryptionError> {
        if master_key.len() != 32 {
            return Err(EncryptionError::InvalidKey(
                "Master key must be 32 bytes (256 bits)".into(),
            ));
        }
        
        let mut key = [0u8; 32];
        key.copy_from_slice(master_key);
        
        let mek_cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| EncryptionError::InvalidKey(e.to_string()))?;
        
        Ok(Self {
            master_key: key,
            mek_cipher,
        })
    }
    
    /// Create KeyManager from hex string
    pub fn from_hex(hex_key: &str) -> Result<Self, EncryptionError> {
        let key = hex::decode(hex_key)
            .map_err(|e| EncryptionError::InvalidKey(format!("Invalid hex: {}", e)))?;
        Self::new(&key)
    }
    
    /// Create KeyManager from passphrase (derives key using SHA-256)
    pub fn from_passphrase(passphrase: &str) -> Result<Self, EncryptionError> {
        let mut hasher = Sha256::new();
        hasher.update(passphrase.as_bytes());
        let key = hasher.finalize();
        Self::new(&key)
    }
    
    /// Generate a new random Data Encryption Key
    pub fn generate_dek(&self) -> [u8; 32] {
        let mut dek = [0u8; 32];
        OsRng.fill_bytes(&mut dek);
        dek
    }
    
    /// Encrypt DEK with Master Key (envelope encryption)
    pub fn encrypt_dek(&self, dek: &[u8; 32]) -> Result<(Vec<u8>, Vec<u8>), EncryptionError> {
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let encrypted_dek = self
            .mek_cipher
            .encrypt(nonce, dek.as_ref())
            .map_err(|e| EncryptionError::EncryptionFailed(e.to_string()))?;
        
        Ok((encrypted_dek, nonce_bytes.to_vec()))
    }
    
    /// Decrypt DEK with Master Key
    pub fn decrypt_dek(&self, encrypted_dek: &[u8], nonce: &[u8]) -> Result<[u8; 32], EncryptionError> {
        if nonce.len() != 12 {
            return Err(EncryptionError::InvalidKey("Nonce must be 12 bytes".into()));
        }
        
        let nonce = Nonce::from_slice(nonce);
        
        let dek = self
            .mek_cipher
            .decrypt(nonce, encrypted_dek)
            .map_err(|e| EncryptionError::DecryptionFailed(e.to_string()))?;
        
        if dek.len() != 32 {
            return Err(EncryptionError::DecryptionFailed("Invalid DEK length".into()));
        }
        
        let mut result = [0u8; 32];
        result.copy_from_slice(&dek);
        Ok(result)
    }
}

/// Object Encryptor for encrypting/decrypting object data
pub struct ObjectEncryptor {
    /// Data Encryption Key
    dek: [u8; 32],
    /// Cipher instance
    cipher: Aes256Gcm,
}

impl ObjectEncryptor {
    /// Create new encryptor with DEK
    pub fn new(dek: &[u8; 32]) -> Result<Self, EncryptionError> {
        let cipher = Aes256Gcm::new_from_slice(dek)
            .map_err(|e| EncryptionError::InvalidKey(e.to_string()))?;
        
        Ok(Self {
            dek: *dek,
            cipher,
        })
    }
    
    /// Create encryptor from customer-provided key (SSE-C)
    pub fn from_customer_key(key_base64: &str) -> Result<(Self, String), EncryptionError> {
        use base64::{Engine as _, engine::general_purpose::STANDARD};
        
        let key = STANDARD.decode(key_base64)
            .map_err(|e| EncryptionError::InvalidKey(format!("Invalid base64: {}", e)))?;
        
        if key.len() != 32 {
            return Err(EncryptionError::InvalidKey(
                "Customer key must be 32 bytes (256 bits)".into(),
            ));
        }
        
        // Calculate MD5 of customer key for verification
        let key_md5 = md5::compute(&key);
        let key_md5_base64 = STANDARD.encode(key_md5.as_ref());
        
        let mut dek = [0u8; 32];
        dek.copy_from_slice(&key);
        
        let encryptor = Self::new(&dek)?;
        Ok((encryptor, key_md5_base64))
    }
    
    /// Encrypt data chunk
    pub fn encrypt(&self, data: &[u8]) -> Result<(Vec<u8>, Vec<u8>), EncryptionError> {
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let ciphertext = self
            .cipher
            .encrypt(nonce, data)
            .map_err(|e| EncryptionError::EncryptionFailed(e.to_string()))?;
        
        Ok((ciphertext, nonce_bytes.to_vec()))
    }
    
    /// Decrypt data chunk
    pub fn decrypt(&self, ciphertext: &[u8], nonce: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        if nonce.len() != 12 {
            return Err(EncryptionError::InvalidKey("Nonce must be 12 bytes".into()));
        }
        
        let nonce = Nonce::from_slice(nonce);
        
        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| EncryptionError::DecryptionFailed(e.to_string()))?;
        
        Ok(plaintext)
    }
    
    /// Generate random nonce
    pub fn generate_nonce() -> [u8; 12] {
        let mut nonce = [0u8; 12];
        OsRng.fill_bytes(&mut nonce);
        nonce
    }
}

/// Streaming encryptor for large objects
pub struct StreamingEncryptor {
    key_manager: Arc<KeyManager>,
    chunk_size: usize,
}

impl StreamingEncryptor {
    pub fn new(key_manager: Arc<KeyManager>, chunk_size: usize) -> Self {
        Self {
            key_manager,
            chunk_size,
        }
    }
    
    /// Encrypt a stream of data, returns encrypted chunks with metadata
    pub fn encrypt_stream(
        &self,
        data: &[u8],
    ) -> Result<(Vec<u8>, EncryptedObjectInfo), EncryptionError> {
        // Generate DEK for this object
        let dek = self.key_manager.generate_dek();
        
        // Encrypt DEK with MEK
        let (encrypted_dek, dek_nonce) = self.key_manager.encrypt_dek(&dek)?;
        
        // Create object encryptor
        let encryptor = ObjectEncryptor::new(&dek)?;
        
        // Encrypt data
        let (ciphertext, data_nonce) = encryptor.encrypt(data)?;
        
        let info = EncryptedObjectInfo {
            sse_type: SseType::SseS3,
            encrypted_dek: Some(encrypted_dek),
            dek_nonce: Some(dek_nonce),
            data_nonce,
            sse_customer_key_md5: None,
        };
        
        Ok((ciphertext, info))
    }
    
    /// Decrypt a stream of data using stored metadata
    pub fn decrypt_stream(
        &self,
        ciphertext: &[u8],
        info: &EncryptedObjectInfo,
    ) -> Result<Vec<u8>, EncryptionError> {
        // Decrypt DEK with MEK
        let encrypted_dek = info
            .encrypted_dek
            .as_ref()
            .ok_or_else(|| EncryptionError::DecryptionFailed("Missing encrypted DEK".into()))?;
        
        let dek_nonce = info
            .dek_nonce
            .as_ref()
            .ok_or_else(|| EncryptionError::DecryptionFailed("Missing DEK nonce".into()))?;
        
        let dek = self.key_manager.decrypt_dek(encrypted_dek, dek_nonce)?;
        
        // Create object encryptor
        let encryptor = ObjectEncryptor::new(&dek)?;
        
        // Decrypt data
        encryptor.decrypt(ciphertext, &info.data_nonce)
    }
}

/// SSE-C encryption helper
pub struct SseCEncryptor;

impl SseCEncryptor {
    /// Encrypt with customer-provided key
    pub fn encrypt(
        data: &[u8],
        customer_key_base64: &str,
    ) -> Result<(Vec<u8>, EncryptedObjectInfo), EncryptionError> {
        let (encryptor, key_md5) = ObjectEncryptor::from_customer_key(customer_key_base64)?;
        
        let (ciphertext, data_nonce) = encryptor.encrypt(data)?;
        
        let info = EncryptedObjectInfo {
            sse_type: SseType::SseC,
            encrypted_dek: None,
            dek_nonce: None,
            data_nonce,
            sse_customer_key_md5: Some(key_md5),
        };
        
        Ok((ciphertext, info))
    }
    
    /// Decrypt with customer-provided key
    pub fn decrypt(
        ciphertext: &[u8],
        info: &EncryptedObjectInfo,
        customer_key_base64: &str,
    ) -> Result<Vec<u8>, EncryptionError> {
        let (encryptor, key_md5) = ObjectEncryptor::from_customer_key(customer_key_base64)?;
        
        // Verify key MD5 matches
        if let Some(ref stored_md5) = info.sse_customer_key_md5 {
            if stored_md5 != &key_md5 {
                return Err(EncryptionError::InvalidKey(
                    "Customer key does not match".into(),
                ));
            }
        }
        
        encryptor.decrypt(ciphertext, &info.data_nonce)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_key_manager() {
        let km = KeyManager::from_passphrase("test-master-key").unwrap();
        
        // Generate and encrypt DEK
        let dek = km.generate_dek();
        let (encrypted_dek, nonce) = km.encrypt_dek(&dek).unwrap();
        
        // Decrypt DEK
        let decrypted_dek = km.decrypt_dek(&encrypted_dek, &nonce).unwrap();
        
        assert_eq!(dek, decrypted_dek);
    }
    
    #[test]
    fn test_object_encryption() {
        let mut dek = [0u8; 32];
        OsRng.fill_bytes(&mut dek);
        
        let encryptor = ObjectEncryptor::new(&dek).unwrap();
        
        let data = b"Hello, World! This is a test message for encryption.";
        let (ciphertext, nonce) = encryptor.encrypt(data).unwrap();
        
        // Ciphertext should be different from plaintext
        assert_ne!(ciphertext.as_slice(), data);
        
        // Decrypt should return original data
        let decrypted = encryptor.decrypt(&ciphertext, &nonce).unwrap();
        assert_eq!(decrypted.as_slice(), data);
    }
    
    #[test]
    fn test_streaming_encryptor() {
        let km = Arc::new(KeyManager::from_passphrase("test-key").unwrap());
        let encryptor = StreamingEncryptor::new(km.clone(), 64 * 1024);
        
        let data = b"Test data for streaming encryption";
        let (ciphertext, info) = encryptor.encrypt_stream(data).unwrap();
        
        assert_eq!(info.sse_type, SseType::SseS3);
        assert!(info.encrypted_dek.is_some());
        
        let decrypted = encryptor.decrypt_stream(&ciphertext, &info).unwrap();
        assert_eq!(decrypted.as_slice(), data);
    }
    
    #[test]
    fn test_sse_c() {
        use base64::{Engine as _, engine::general_purpose::STANDARD};
        
        // Generate random customer key
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        let key_base64 = STANDARD.encode(&key);
        
        let data = b"Customer encrypted data";
        let (ciphertext, info) = SseCEncryptor::encrypt(data, &key_base64).unwrap();
        
        assert_eq!(info.sse_type, SseType::SseC);
        assert!(info.sse_customer_key_md5.is_some());
        
        let decrypted = SseCEncryptor::decrypt(&ciphertext, &info, &key_base64).unwrap();
        assert_eq!(decrypted.as_slice(), data);
    }
}
