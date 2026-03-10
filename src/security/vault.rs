use crate::error::{OraError, Result};
use crate::security::crypto::{decrypt_data, derive_key, encrypt_data, HardwareFingerprint};
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credential {
    pub provider: String,
    pub api_key: String,
    pub endpoint: Option<String>,
    pub org_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl Credential {
    pub fn new(provider: String, api_key: String) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            provider,
            api_key,
            endpoint: None,
            org_id: None,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultMetadata {
    pub version: String,
    pub created_at: String,
    pub updated_at: String,
    pub salt: String,
    pub checksum: String,
}

impl Default for VaultMetadata {
    fn default() -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        let salt_bytes: [u8; 32] = rand::random();
        Self {
            version: "2.0".to_string(),
            created_at: now.clone(),
            updated_at: now,
            salt: base64::engine::general_purpose::STANDARD.encode(salt_bytes),
            checksum: String::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Vault {
    path: PathBuf,
    unlocked: bool,
    credentials: std::collections::HashMap<String, Credential>,
    metadata: VaultMetadata,
    master_key: Option<Vec<u8>>,
}

impl Vault {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            unlocked: false,
            credentials: std::collections::HashMap::new(),
            metadata: VaultMetadata::default(),
            master_key: None,
        }
    }

    pub fn exists(&self) -> bool {
        self.path.exists()
    }
    pub fn is_unlocked(&self) -> bool {
        self.unlocked
    }

    pub fn create(&mut self, password: Option<&str>) -> Result<bool> {
        if self.exists() {
            return Ok(false);
        }
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        self.metadata = VaultMetadata::default();
        self.credentials.clear();
        self.unlocked = true;

        let (key_material, salt) = HardwareFingerprint::derive_key_material(password);
        self.master_key = Some(derive_key(&key_material, &salt)?);
        self.save()?;
        Ok(true)
    }

    pub fn unlock(&mut self, password: Option<&str>) -> Result<bool> {
        if !self.exists() {
            return self.create(password);
        }

        let data = std::fs::read(&self.path)?;
        let (metadata_json, encrypted_blob) = Self::parse_vault_file(&data)?;
        self.metadata =
            serde_json::from_str(&metadata_json).map_err(|_| OraError::VaultCorrupted)?;

        let (key_material, _) = HardwareFingerprint::derive_key_material(password);
        let key = derive_key(&key_material, self.metadata.salt.as_bytes())?;

        let decrypted = decrypt_data(&encrypted_blob, &key)?;
        self.credentials =
            serde_json::from_slice(&decrypted).map_err(|_| OraError::VaultCorrupted)?;

        self.master_key = Some(key);
        self.unlocked = true;
        Ok(true)
    }

    pub fn lock(&mut self) {
        self.credentials.clear();
        self.master_key = None;
        self.unlocked = false;
    }

    pub fn set(&mut self, provider: &str, credential: Credential) -> Result<bool> {
        if !self.unlocked {
            return Err(OraError::VaultLocked);
        }
        self.credentials.insert(provider.to_string(), credential);
        self.save()?;
        Ok(true)
    }

    pub fn get(&self, provider: &str) -> Option<&Credential> {
        if !self.unlocked {
            return None;
        }
        self.credentials.get(provider)
    }

    pub fn list_providers(&self) -> Vec<String> {
        if !self.unlocked {
            return Vec::new();
        }
        self.credentials.keys().cloned().collect()
    }

    pub fn delete(&mut self, provider: &str) -> Result<bool> {
        if !self.unlocked {
            return Err(OraError::VaultLocked);
        }
        if self.credentials.remove(provider).is_some() {
            self.save()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn save(&self) -> Result<()> {
        if !self.unlocked || self.master_key.is_none() {
            return Err(OraError::VaultLocked);
        }

        let key = self.master_key.as_ref().unwrap();
        let data = serde_json::to_vec(&self.credentials)?;
        let encrypted = encrypt_data(&data, key)?;

        let mut metadata = self.metadata.clone();
        metadata.updated_at = chrono::Utc::now().to_rfc3339();
        metadata.checksum = Self::compute_checksum(&self.credentials);

        let metadata_json = serde_json::to_string(&metadata)?;
        let output = format!(
            "{}\n---VAULT_DATA---\n{}",
            metadata_json,
            base64::engine::general_purpose::STANDARD.encode(&encrypted)
        );

        std::fs::write(&self.path, output)?;
        Ok(())
    }

    fn parse_vault_file(data: &[u8]) -> Result<(String, Vec<u8>)> {
        let content = String::from_utf8_lossy(data);
        if let Some(pos) = content.find("\n---VAULT_DATA---\n") {
            let metadata = content[..pos].to_string();
            let encrypted_b64 = content[pos + "\n---VAULT_DATA---\n".len()..].trim();
            let encrypted = base64::engine::general_purpose::STANDARD
                .decode(encrypted_b64)
                .map_err(|_| OraError::VaultCorrupted)?;
            Ok((metadata, encrypted))
        } else {
            Err(OraError::VaultCorrupted)
        }
    }

    fn compute_checksum(credentials: &std::collections::HashMap<String, Credential>) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        let data = serde_json::to_string(credentials).unwrap_or_default();
        hasher.update(data.as_bytes());
        hex::encode(hasher.finalize())[..16].to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;

    #[test]
    fn test_vault_operations() {
        let path = temp_dir().join("ora_vault_test.dat");
        let mut vault = Vault::new(path.clone());
        let created = vault.create(None);
        assert!(created.is_ok());
        let cred = Credential::new("openai".to_string(), "sk-test123".to_string());
        let _ = vault.set("openai", cred);
        let retrieved = vault.get("openai");
        assert!(retrieved.is_some());
        vault.lock();
        assert!(!vault.is_unlocked());
        let _ = std::fs::remove_file(path);
    }
}
