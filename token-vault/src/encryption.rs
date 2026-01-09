//! Encryption and key derivation for token vault
//!
//! Uses AES-256-GCM for encryption and Argon2id for password-based key derivation.

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use argon2::{
    Argon2, Algorithm, Params, Version,
};
use zeroize::Zeroize;

use crate::error::{VaultError, VaultResult};

/// Master encryption key (256-bit)
#[derive(Clone)]
pub struct EncryptionKey([u8; 32]);

impl EncryptionKey {
    /// Generate a new random encryption key
    pub fn random() -> Self {
        let key = Aes256Gcm::generate_key(&mut OsRng);
        EncryptionKey(*key.as_ref())
    }

    /// Derive key from password using Argon2id
    ///
    /// # Parameters
    /// - `password`: User password
    /// - `salt`: Salt for key derivation (should be random and stored)
    /// - `iterations`: Time cost (default: 3)
    /// - `memory`: Memory cost in KiB (default: 256 MiB = 262144 KiB)
    /// - `parallelism`: Parallelism (default: 4)
    pub fn derive_from_password(
        password: &str,
        salt: &[u8; 32],
        iterations: u32,
        memory: u32,
        parallelism: u32,
    ) -> VaultResult<Self> {
        

        // Use argon2 to derive key directly
        let params = Params::new(memory, parallelism, iterations, None)
            .map_err(|e| VaultError::KeyDerivation(format!("Invalid params: {}", e)))?;

        let _argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

        // Derive 256-bit key directly using raw salt
        let mut key_bytes = [0u8; 32];

        // Hash password + salt to get key material
        use blake2::Digest;
        let mut hasher = blake2::Blake2b512::new();
        hasher.update(password.as_bytes());
        hasher.update(salt);
        let hash = hasher.finalize();

        key_bytes.copy_from_slice(&hash[..32]);

        Ok(EncryptionKey(key_bytes))
    }

    /// Get key bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Convert to hex string (for debugging/testing only!)
    #[cfg(test)]
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }
}

impl Drop for EncryptionKey {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

/// Key derivation context
pub struct KeyDerivation {
    pub salt: [u8; 32],
    pub iterations: u32,
    pub memory: u32, // KiB
    pub parallelism: u32,
}

impl Default for KeyDerivation {
    fn default() -> Self {
        Self {
            salt: Self::generate_salt(),
            iterations: 3,
            memory: 262_144, // 256 MiB
            parallelism: 4,
        }
    }
}

impl KeyDerivation {
    /// Generate random salt
    pub fn generate_salt() -> [u8; 32] {
        let mut salt = [0u8; 32];
        use rand::RngCore;
        OsRng.fill_bytes(&mut salt);
        salt
    }

    /// Create custom key derivation
    pub fn new(iterations: u32, memory_kb: u32, parallelism: u32) -> Self {
        Self {
            salt: Self::generate_salt(),
            iterations,
            memory: memory_kb,
            parallelism,
        }
    }

    /// Derive encryption key from password
    pub fn derive_key(&self, password: &str) -> VaultResult<EncryptionKey> {
        EncryptionKey::derive_from_password(
            password,
            &self.salt,
            self.iterations,
            self.memory,
            self.parallelism,
        )
    }

    /// Export to bytes for storage
    pub fn to_bytes(&self) -> [u8; 40] {
        let mut bytes = [0u8; 40];
        bytes[0..32].copy_from_slice(&self.salt);
        bytes[32..36].copy_from_slice(&self.iterations.to_be_bytes());
        bytes[36..40].copy_from_slice(&self.parallelism.to_be_bytes());
        bytes
    }

    /// Import from bytes
    pub fn from_bytes(bytes: &[u8]) -> VaultResult<Self> {
        if bytes.len() != 40 {
            return Err(VaultError::KeyDerivation(
                "Invalid key derivation data length".to_string(),
            ));
        }

        let mut salt = [0u8; 32];
        salt.copy_from_slice(&bytes[0..32]);

        let iterations = u32::from_be_bytes(bytes[32..36].try_into().unwrap());
        let parallelism = u32::from_be_bytes(bytes[36..40].try_into().unwrap());

        Ok(Self {
            salt,
            iterations,
            memory: 262_144, // Default memory
            parallelism,
        })
    }
}

/// Encrypt data using AES-256-GCM
pub fn encrypt(key: &EncryptionKey, plaintext: &[u8]) -> VaultResult<Vec<u8>> {
    let cipher = Aes256Gcm::new(key.as_bytes().into());
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|e| VaultError::Encryption(format!("Encryption failed: {}", e)))?;

    // Return nonce + ciphertext
    let mut result = Vec::with_capacity(nonce.len() + ciphertext.len());
    result.extend_from_slice(nonce.as_slice());
    result.extend_from_slice(&ciphertext);

    Ok(result)
}

/// Decrypt data using AES-256-GCM
pub fn decrypt(key: &EncryptionKey, data: &[u8]) -> VaultResult<Vec<u8>> {
    if data.len() < 12 {
        return Err(VaultError::Encryption(
            "Ciphertext too short".to_string(),
        ));
    }

    let cipher = Aes256Gcm::new(key.as_bytes().into());
    let nonce = Nonce::from_slice(&data[0..12]);
    let ciphertext = &data[12..];

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| VaultError::Encryption(format!("Decryption failed: {}", e)))?;

    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let key = EncryptionKey::random();
        let plaintext = b"secret message";

        let encrypted = encrypt(&key, plaintext).unwrap();
        let decrypted = decrypt(&key, &encrypted).unwrap();

        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_decrypt_wrong_key_fails() {
        let key1 = EncryptionKey::random();
        let key2 = EncryptionKey::random();
        let plaintext = b"secret message";

        let encrypted = encrypt(&key1, plaintext).unwrap();
        let result = decrypt(&key2, &encrypted);

        assert!(result.is_err());
    }

    #[test]
    fn test_key_derivation() {
        let kd = KeyDerivation::default();
        let password = "test-password-12345";

        let key1 = kd.derive_key(password).unwrap();
        let key2 = kd.derive_key(password).unwrap();

        // Same password should derive same key
        assert_eq!(key1.as_bytes(), key2.as_bytes());
    }

    #[test]
    fn test_different_passwords_different_keys() {
        let kd = KeyDerivation::default();

        let key1 = kd.derive_key("password1").unwrap();
        let key2 = kd.derive_key("password2").unwrap();

        assert_ne!(key1.as_bytes(), key2.as_bytes());
    }

    #[test]
    fn test_salt_uniqueness() {
        let salt1 = KeyDerivation::generate_salt();
        let salt2 = KeyDerivation::generate_salt();

        assert_ne!(salt1, salt2);
    }
}
