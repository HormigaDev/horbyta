use crate::utils::validate_password;
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize, Clone, Validate, ToSchema)]
pub struct RegisterPayload {
    #[validate(length(min = 2, max = 100, message = "name must be between 2 and 100 characters"))]
    pub name: String,
    #[validate(email(message = "email must be a valid email address"))]
    pub email: String,
    #[validate(custom(function = "validate_password"))]
    pub password: String,
}

#[derive(Deserialize, Clone, Validate, ToSchema)]
pub struct LoginPayload {
    #[validate(email(message = "email must be a valid email address"))]
    pub email: String,
    #[validate(length(min = 1, message = "password is required"))]
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

#[derive(Deserialize, Clone, Validate, ToSchema)]
pub struct ClientLoginPayload {
    pub user_id: Uuid,
    #[validate(email(message = "email must be a valid email address"))]
    pub email: String,
    #[validate(length(min = 1, message = "password is required"))]
    pub password: String,
}

#[derive(Deserialize, Clone, Validate, ToSchema)]
pub struct UpdatePassword {
    #[validate(length(min = 1, message = "current password is required"))]
    pub current_password: String,
    #[validate(custom(function = "validate_password"))]
    pub new_password: String,
}

#[derive(Deserialize, Clone, Validate, ToSchema)]
pub struct ClientMagicTokenLogin {
    pub user_id: Uuid,
    #[validate(length(min = 1, message = "magic token is required"))]
    pub magic_token: String,
}
