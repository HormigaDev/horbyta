use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::{Validate, ValidationError};

fn validate_one_two_factor_code(payload: &DisableTotpDto) -> Result<(), ValidationError> {
    match (
        payload.two_factor_auth_code.as_ref(),
        payload.two_factor_auth_backup_code.as_ref(),
    ) {
        (Some(_), Some(_)) => Err(ValidationError::new("multiple_two_factor_auth_codes")),
        (None, None) => Err(ValidationError::new("two_factor_auth_code_required")),
        _ => Ok(()),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TotpSetupDto {
    pub otpauth_url: String,
    pub qr_png_base64: String,
    pub qr_data_uri: String,
    pub manual_entry_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TotpStatusDto {
    pub enabled: bool,
    pub setup_pending: bool,
    pub backup_codes_remaining: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TotpBackupCodesDto {
    pub backup_codes: Vec<String>,
    pub backup_codes_file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TotpEnableDto {
    pub status: TotpStatusDto,
    pub backup_codes: Vec<String>,
    pub backup_codes_file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct VerifyTotpCodeDto {
    #[validate(length(
        min = 6,
        max = 8,
        message = "two-factor authentication code must be between 6 and 8 digits"
    ))]
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
#[serde(deny_unknown_fields)]
#[validate(schema(function = "validate_one_two_factor_code"))]
pub struct DisableTotpDto {
    #[validate(length(min = 1, max = 128, message = "password is required"))]
    pub password: String,

    #[validate(length(
        min = 6,
        max = 8,
        message = "two-factor authentication code must be between 6 and 8 digits"
    ))]
    pub two_factor_auth_code: Option<String>,

    #[validate(length(
        min = 8,
        max = 32,
        message = "two-factor authentication backup code must be between 8 and 32 characters"
    ))]
    pub two_factor_auth_backup_code: Option<String>,
}
