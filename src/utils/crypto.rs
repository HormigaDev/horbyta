use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use jsonwebtoken::{EncodingKey, Header, encode};
use rand::RngExt;
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::error::AppError;

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
