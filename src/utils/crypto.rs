use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit, OsRng, rand_core::RngCore},
};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::SaltString};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use jsonwebtoken::{EncodingKey, Header, encode};
use rand::RngExt;
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::error::AppError;

const NONCE_LENGTH: usize = 12;
const KEY_LENGTH: usize = 32;

fn decode_key(key_b64: &str) -> Result<[u8; KEY_LENGTH], AppError> {
    let raw = STANDARD
        .decode(key_b64)
        .map_err(|_| AppError::Internal("ai_prefs_encryption_key is not valid base64".to_string()))?;

    if raw.len() != KEY_LENGTH {
        return Err(AppError::Internal(
            "ai_prefs_encryption_key must represent 32 bytes (aes-256-gcm)".to_string(),
        ));
    }

    let mut key = [0u8; KEY_LENGTH];
    key.copy_from_slice(&raw);
    Ok(key)
}

pub fn encrypt_secret(key_b64: &str, plaintext: &str) -> Result<String, AppError> {
    let key = decode_key(key_b64)?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|_| AppError::Internal("could not initialize aes encryption".to_string()))?;

    let mut nonce_bytes = [0u8; NONCE_LENGTH];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|_| AppError::Internal("could not encrypt secret".to_string()))?;

    let mut packed = Vec::with_capacity(NONCE_LENGTH + ciphertext.len());
    packed.extend_from_slice(&nonce_bytes);
    packed.extend_from_slice(&ciphertext);

    Ok(STANDARD.encode(packed))
}

pub fn decrypt_secret(key_b64: &str, encrypted_b64: &str) -> Result<String, AppError> {
    let key = decode_key(key_b64)?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|_| AppError::Internal("could not initialize aes decryption".to_string()))?;

    let packed = STANDARD
        .decode(encrypted_b64)
        .map_err(|_| AppError::Internal("invalid encrypted secret (base64)".to_string()))?;

    if packed.len() <= NONCE_LENGTH {
        return Err(AppError::Internal("invalid encrypted secret (size)".to_string()));
    }

    let (nonce_bytes, ciphertext) = packed.split_at(NONCE_LENGTH);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext_bytes = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| AppError::Internal("could not decrypt secret".to_string()))?;

    String::from_utf8(plaintext_bytes)
        .map_err(|_| AppError::Internal("decrypted secret is not valid utf-8".to_string()))
}

pub fn hash_password(password: String) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default().hash_password(password.as_bytes(), &salt)?.to_string();

    Ok(password_hash)
}

pub fn compare_password(password: String, hashed_password: String) -> Result<bool, AppError> {
    let parsed_hash = PasswordHash::new(&hashed_password)?;

    if Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_err()
    {
        return Err(AppError::Unauthorized("invalid credentials".to_string()));
    } else {
        return Ok(true);
    }
}

pub fn generate_magic_link_token() -> (String, Vec<u8>) {
    let mut rng = rand::rng();

    let token_bytes: [u8; 32] = rng.random();
    let token = hex::encode(token_bytes);
    let hash = Sha256::digest(token_bytes);

    (token, hash.to_vec())
}

pub fn verify_magic_link_token(token: &str) -> Result<Vec<u8>, AppError> {
    let token_bytes = hex::decode(token).map_err(|_| AppError::Unauthorized("invalid token".to_string()))?;

    let hash = Sha256::digest(token_bytes);

    Ok(hash.to_vec())
}

pub fn generate_jwt<T: Serialize>(claims: &T, secret: &str) -> Result<String, AppError> {
    encode(&Header::default(), claims, &EncodingKey::from_secret(secret.as_bytes()))
        .map_err(|e| AppError::Internal(e.to_string()))
}
