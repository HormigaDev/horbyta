use constant_time_eq::constant_time_eq;
use rand::RngExt;
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};
use totp_rs::{Algorithm, Secret, TOTP};

use crate::{dtos::totp_2fa::TotpSetupDto, error::Totp2FaError};

#[derive(Debug, Clone)]
pub struct TotpPolicy {
    pub issuer: String,
    pub digits: usize,
    pub skew: u8,
    pub step: u64,
}

impl TotpPolicy {
    pub fn new(issuer: impl Into<String>) -> Self {
        Self {
            issuer: issuer.into(),
            digits: 6,
            skew: 1,
            step: 30,
        }
    }
}

pub fn default_totp_policy() -> TotpPolicy {
    TotpPolicy::new("Horbyta")
}

#[derive(Debug, Clone)]
pub struct TotpSetup {
    pub secret_base32: String,
    pub dto: TotpSetupDto,
}

#[derive(Debug, Clone)]
pub struct TotpVerification {
    pub matched_step: u64,
    pub new_last_used_step: i64,
}

#[derive(Debug, Clone)]
pub struct BackupCodeVerification {
    pub remaining_hashes: Vec<String>,
}

pub fn generate_setup(policy: &TotpPolicy, account_name: &str) -> Result<TotpSetup, Totp2FaError> {
    let secret_base32 = Secret::generate_secret().to_encoded().to_string();
    let totp = build_totp(policy, account_name, &secret_base32)?;
    let otpauth_url = totp.get_url();
    let qr_png_base64 = totp.get_qr_base64().map_err(|_| Totp2FaError::QrGeneration)?;
    let qr_data_uri = format!("data:image/png;base64,{qr_png_base64}");

    Ok(TotpSetup {
        secret_base32: secret_base32.clone(),
        dto: TotpSetupDto {
            otpauth_url,
            qr_png_base64,
            qr_data_uri,
            manual_entry_key: secret_base32,
        },
    })
}

pub fn generate_backup_codes() -> Vec<String> {
    let mut rng = rand::rng();

    (0..10)
        .map(|_| {
            let bytes: [u8; 8] = rng.random();
            let hex = hex::encode_upper(bytes);
            format!("{}-{}-{}-{}", &hex[0..4], &hex[4..8], &hex[8..12], &hex[12..16])
        })
        .collect()
}

pub fn hash_backup_code(code: &str) -> Result<String, Totp2FaError> {
    let normalized = normalize_backup_code(code)?;
    Ok(hex::encode(Sha256::digest(normalized.as_bytes())))
}

pub fn hash_backup_codes(codes: &[String]) -> Result<Vec<String>, Totp2FaError> {
    codes.iter().map(|code| hash_backup_code(code)).collect()
}

pub fn backup_codes_file_content(codes: &[String]) -> String {
    let mut content = "Horbyta two-factor authentication backup codes\n\n".to_string();
    content.push_str("Store these codes in a secure place. Each code can be used only once.\n\n");
    content.push_str(&codes.join("\n"));
    content.push('\n');
    content
}

pub fn verify_code(
    policy: &TotpPolicy,
    account_name: &str,
    secret_base32: &str,
    code: &str,
    last_used_step: Option<i64>,
) -> Result<TotpVerification, Totp2FaError> {
    let code = code.trim();
    if !code.chars().all(|ch| ch.is_ascii_digit()) {
        return Err(Totp2FaError::InvalidCode);
    }

    let totp = build_totp(policy, account_name, secret_base32)?;
    let current_step = current_unix_time()? / policy.step;
    let first_step = current_step.saturating_sub(policy.skew as u64);
    let last_step = current_step + policy.skew as u64;

    for step in first_step..=last_step {
        let token = totp.generate(step * policy.step);
        if constant_time_eq(token.as_bytes(), code.as_bytes()) {
            let step_i64 = i64::try_from(step).map_err(|_| Totp2FaError::InvalidCode)?;
            if last_used_step.is_some_and(|last| step_i64 <= last) {
                return Err(Totp2FaError::ReplayedCode);
            }

            return Ok(TotpVerification {
                matched_step: step,
                new_last_used_step: step_i64,
            });
        }
    }

    Err(Totp2FaError::InvalidCode)
}

pub fn verify_backup_code(code: &str, stored_hashes: &[String]) -> Result<BackupCodeVerification, Totp2FaError> {
    let hash = hash_backup_code(code)?;
    let mut matched_index = None;

    for (index, stored_hash) in stored_hashes.iter().enumerate() {
        if constant_time_eq(stored_hash.as_bytes(), hash.as_bytes()) {
            matched_index = Some(index);
            break;
        }
    }

    let matched_index = matched_index.ok_or(Totp2FaError::InvalidCode)?;
    let mut remaining_hashes = stored_hashes.to_vec();
    remaining_hashes.remove(matched_index);

    Ok(BackupCodeVerification { remaining_hashes })
}

fn build_totp(policy: &TotpPolicy, account_name: &str, secret_base32: &str) -> Result<TOTP, Totp2FaError> {
    let secret = Secret::Encoded(secret_base32.to_string())
        .to_bytes()
        .map_err(|_| Totp2FaError::InvalidSecret)?;

    TOTP::new(
        Algorithm::SHA1,
        policy.digits,
        policy.skew,
        policy.step,
        secret,
        Some(policy.issuer.clone()),
        account_name.to_string(),
    )
    .map_err(|_| Totp2FaError::InvalidSecret)
}

fn current_unix_time() -> Result<u64, Totp2FaError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|_| Totp2FaError::InvalidCode)
}

fn normalize_backup_code(code: &str) -> Result<String, Totp2FaError> {
    let normalized: String = code
        .chars()
        .filter(|ch| !ch.is_whitespace() && *ch != '-')
        .flat_map(char::to_uppercase)
        .collect();

    if normalized.len() < 8 || !normalized.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err(Totp2FaError::InvalidCode);
    }

    Ok(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backup_code_verification_consumes_code_once() {
        let codes = vec!["ABCD-1234-EF56-7890".to_string()];
        let hashes = hash_backup_codes(&codes).expect("backup code should hash");

        let verification = verify_backup_code(&codes[0], &hashes).expect("backup code should verify");
        assert!(verification.remaining_hashes.is_empty());

        let replay = verify_backup_code(&codes[0], &verification.remaining_hashes);
        assert!(matches!(replay, Err(Totp2FaError::InvalidCode)));
    }

    #[test]
    fn backup_code_verification_accepts_normalized_input() {
        let codes = vec!["ABCD-1234-EF56-7890".to_string()];
        let hashes = hash_backup_codes(&codes).expect("backup code should hash");

        let verification = verify_backup_code("abcd1234 ef567890", &hashes);
        assert!(verification.is_ok());
    }
}
