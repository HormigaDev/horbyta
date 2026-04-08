use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit, OsRng, rand_core::RngCore},
};
use base64::{Engine as _, engine::general_purpose::STANDARD};

use crate::error::AppError;

const NONCE_LENGTH: usize = 12;
const KEY_LENGTH: usize = 32;

fn decode_key(key_b64: &str) -> Result<[u8; KEY_LENGTH], AppError> {
    let raw = STANDARD
        .decode(key_b64)
        .map_err(|_| AppError::Internal("AI_PREFS_ENCRYPTION_KEY no es base64 valido".to_string()))?;

    if raw.len() != KEY_LENGTH {
        return Err(AppError::Internal(
            "AI_PREFS_ENCRYPTION_KEY debe representar 32 bytes (AES-256-GCM)".to_string(),
        ));
    }

    let mut key = [0u8; KEY_LENGTH];
    key.copy_from_slice(&raw);
    Ok(key)
}

pub fn encrypt_secret(key_b64: &str, plaintext: &str) -> Result<String, AppError> {
    let key = decode_key(key_b64)?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|_| AppError::Internal("No se pudo inicializar cifrado AES".to_string()))?;

    let mut nonce_bytes = [0u8; NONCE_LENGTH];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|_| AppError::Internal("No se pudo cifrar el secreto".to_string()))?;

    let mut packed = Vec::with_capacity(NONCE_LENGTH + ciphertext.len());
    packed.extend_from_slice(&nonce_bytes);
    packed.extend_from_slice(&ciphertext);

    Ok(STANDARD.encode(packed))
}

pub fn decrypt_secret(key_b64: &str, encrypted_b64: &str) -> Result<String, AppError> {
    let key = decode_key(key_b64)?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|_| AppError::Internal("No se pudo inicializar descifrado AES".to_string()))?;

    let packed = STANDARD
        .decode(encrypted_b64)
        .map_err(|_| AppError::Internal("Secreto cifrado invalido (base64)".to_string()))?;

    if packed.len() <= NONCE_LENGTH {
        return Err(AppError::Internal("Secreto cifrado invalido (tamano)".to_string()));
    }

    let (nonce_bytes, ciphertext) = packed.split_at(NONCE_LENGTH);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext_bytes = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| AppError::Internal("No se pudo descifrar el secreto".to_string()))?;

    String::from_utf8(plaintext_bytes)
        .map_err(|_| AppError::Internal("Secreto descifrado no es UTF-8 valido".to_string()))
}
