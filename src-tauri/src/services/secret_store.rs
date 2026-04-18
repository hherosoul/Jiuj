use crate::db::SettingsRepo;
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use rand::RngCore;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

#[derive(Clone)]
pub struct SecretStore {
    encryption_key: Option<[u8; 32]>,
}

impl SecretStore {
    pub fn new(app_data_dir: &str) -> Self {
        let mut key_path = PathBuf::from(app_data_dir);
        key_path.push(".key");

        let encryption_key = Self::load_or_create_key(&key_path).ok();

        SecretStore {
            encryption_key,
        }
    }

    fn load_or_create_key(key_path: &PathBuf) -> Result<[u8; 32], Box<dyn std::error::Error>> {
        if key_path.exists() {
            let key_bytes = fs::read(key_path)?;
            let mut key = [0u8; 32];
            key.copy_from_slice(&key_bytes[..32]);
            return Ok(key);
        }

        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);

        if let Some(parent) = key_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut file = File::create(key_path)?;
        file.write_all(&key)?;
        file.sync_all()?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(key_path, fs::Permissions::from_mode(0o600))?;
        }

        Ok(key)
    }

    fn get_cipher(&self) -> Result<Aes256Gcm, Box<dyn std::error::Error>> {
        let key = self.encryption_key.ok_or("No encryption key available")?;
        Ok(Aes256Gcm::new(&key.into()))
    }

    pub fn set_password(&self, key: &str, value: &str, settings_repo: &SettingsRepo) -> Result<(), Box<dyn std::error::Error>> {
        let cipher = self.get_cipher()?;
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher.encrypt(nonce, value.as_bytes()).map_err(|e| format!("Encryption error: {}", e))?;
        let mut combined = nonce_bytes.to_vec();
        combined.extend_from_slice(&ciphertext);
        let encoded = BASE64.encode(combined);

        settings_repo.set(key, &encoded)?;
        Ok(())
    }

    pub fn get_password(&self, key: &str, settings_repo: &SettingsRepo) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let encoded = match settings_repo.get(key) {
            Some(v) => v,
            None => return Ok(None),
        };

        let cipher = self.get_cipher()?;
        let combined = BASE64.decode(&encoded)?;

        if combined.len() < 12 {
            return Ok(None);
        }

        let nonce = Nonce::from_slice(&combined[..12]);
        let plaintext = cipher.decrypt(nonce, &combined[12..]).map_err(|e| format!("Decryption error: {}", e))?;
        Ok(Some(String::from_utf8(plaintext)?))
    }

    pub fn delete_password(&self, key: &str, settings_repo: &SettingsRepo) -> Result<bool, Box<dyn std::error::Error>> {
        if settings_repo.get(key).is_some() {
            settings_repo.set(key, "")?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
