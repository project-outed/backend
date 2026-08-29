use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use anyhow::{Context, Result};
use base64::{engine::general_purpose, Engine as _};
use std::env;
use rand::RngCore;

#[allow(dead_code)]
pub fn encrypt(data: &str) -> Result<String> {
    let key_str = env::var("ENCRYPTION_KEY").context("ENCRYPTION_KEY must be set in .env (32 bytes)")?;
    let key = key_str.as_bytes();
    
    if key.len() != 32 {
        return Err(anyhow::anyhow!("Encryption key must be exactly 32 bytes"));
    }

    let cipher = Aes256Gcm::new_from_slice(key)?;
    
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher.encrypt(nonce, data.as_bytes())
        .map_err(|e| anyhow::anyhow!("Encryption error: {:?}", e))?;

    let mut combined = nonce_bytes.to_vec();
    combined.extend_from_slice(&ciphertext);

    Ok(general_purpose::STANDARD.encode(combined))
}

#[allow(dead_code)]
pub fn decrypt(encrypted_data: &str) -> Result<String> {
    let key_str = env::var("ENCRYPTION_KEY").context("ENCRYPTION_KEY must be set in .env (32 bytes)")?;
    let key = key_str.as_bytes();

    let combined = general_purpose::STANDARD.decode(encrypted_data)?;
    if combined.len() < 12 {
        return Err(anyhow::anyhow!("Invalid encrypted data: too short"));
    }

    let (nonce_bytes, ciphertext) = combined.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);
    
    let cipher = Aes256Gcm::new_from_slice(key)?;
    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| anyhow::anyhow!("Decryption error: {:?}", e))?;

    Ok(String::from_utf8(plaintext)?)
}
