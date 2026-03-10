//! OrA Cryptography Utilities
//!
//! Cryptographic utilities for the vault and security systems.

use crate::error::{OraError, Result};

/// Derive encryption key from password/hardware
pub fn derive_key(material: &[u8], salt: &[u8]) -> Result<Vec<u8>> {
    use pbkdf2::pbkdf2_hmac;
    use sha2::Sha256;

    let mut key = vec![0u8; 32]; // 256-bit key
    pbkdf2_hmac::<Sha256>(material, salt, 100_000, &mut key);

    Ok(key)
}

/// Encrypt data using AES-256-GCM
pub fn encrypt_data(data: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
    use ring::rand::{SecureRandom, SystemRandom};

    // Create key
    let unbound_key =
        UnboundKey::new(&AES_256_GCM, key).map_err(|_| OraError::EncryptionError {
            message: "Failed to create key".to_string(),
        })?;
    let less_safe_key = LessSafeKey::new(unbound_key);

    // Generate random nonce
    let rng = SystemRandom::new();
    let mut nonce_bytes = [0u8; 12];
    rng.fill(&mut nonce_bytes)
        .map_err(|_| OraError::EncryptionError {
            message: "Failed to generate nonce".to_string(),
        })?;
    let nonce = Nonce::assume_unique_for_key(nonce_bytes);

    // Encrypt
    let mut in_out = data.to_vec();
    less_safe_key
        .seal_in_place_append_tag(nonce, Aad::empty(), &mut in_out)
        .map_err(|_| OraError::EncryptionError {
            message: "Encryption failed".to_string(),
        })?;

    // Prepend nonce
    let mut result = nonce_bytes.to_vec();
    result.extend(in_out);

    Ok(result)
}

/// Decrypt data using AES-256-GCM
pub fn decrypt_data(encrypted: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};

    if encrypted.len() < 12 {
        return Err(OraError::EncryptionError {
            message: "Invalid encrypted data".to_string(),
        });
    }

    // Extract nonce and ciphertext
    let nonce_bytes = &encrypted[..12];
    let ciphertext = &encrypted[12..];

    // Create key
    let unbound_key =
        UnboundKey::new(&AES_256_GCM, key).map_err(|_| OraError::EncryptionError {
            message: "Failed to create key".to_string(),
        })?;
    let less_safe_key = LessSafeKey::new(unbound_key);

    // Create nonce
    let nonce = Nonce::assume_unique_for_key(nonce_bytes.try_into().map_err(|_| {
        OraError::EncryptionError {
            message: "Invalid nonce".to_string(),
        }
    })?);

    // Decrypt
    let mut in_out = ciphertext.to_vec();
    let decrypted = less_safe_key
        .open_in_place(nonce, Aad::empty(), &mut in_out)
        .map_err(|_| OraError::EncryptionError {
            message: "Decryption failed".to_string(),
        })?;

    Ok(decrypted.to_vec())
}

/// Hardware fingerprint for hardware-bound encryption
pub struct HardwareFingerprint;

impl HardwareFingerprint {
    /// Get machine-specific identifier
    pub fn get_machine_id() -> String {
        #[cfg(target_os = "linux")]
        {
            if let Ok(id) = std::fs::read_to_string("/etc/machine-id") {
                return id.trim().to_string();
            }
        }

        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            let output = Command::new("ioreg")
                .args(["-rd1", "-c", "IOPlatformExpertDevice", "-a"])
                .output();

            if let Ok(output) = output {
                let s = String::from_utf8_lossy(&output.stdout);
                if let Some(pos) = s.find("IOPlatformUUID") {
                    let start = s[pos..].find('"').map(|p| pos + p + 1).unwrap_or(pos);
                    let end = s[start..].find('"').map(|p| start + p).unwrap_or(s.len());
                    return s[start..end].to_string();
                }
            }
        }

        // Fallback
        let hostname = std::env::var("HOSTNAME").unwrap_or_else(|_| "unknown".to_string());
        let user = std::env::var("USER").unwrap_or_else(|_| "".to_string());
        format!("{}-{}", hostname, user)
    }

    /// Derive key material from hardware + optional password
    pub fn derive_key_material(password: Option<&str>) -> (Vec<u8>, Vec<u8>) {
        let machine_id = Self::get_machine_id();

        let combined = match password {
            Some(pw) => format!("{}:{}", machine_id, pw),
            None => machine_id.clone(),
        };

        let salt = {
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(machine_id.as_bytes());
            hasher.finalize()[..16].to_vec()
        };

        (combined.into_bytes(), salt)
    }
}

/// Generate a random string
pub fn generate_random_string(length: usize) -> String {
    use base64::Engine;
    use rand::Rng;

    let bytes: Vec<u8> = (0..length).map(|_| rand::thread_rng().gen()).collect();
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

/// Hash data with SHA-256
pub fn sha256(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}
