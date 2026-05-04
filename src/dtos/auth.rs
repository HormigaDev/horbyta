use crate::utils::validate_password;
use serde::Deserialize;
use utoipa::ToSchema;
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
    #[validate(length(min = 1, message = "magic token is required"))]
    pub magic_token: String,
}
