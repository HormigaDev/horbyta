use crate::utils::validate_password;
use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, Clone, Validate)]
pub struct LoginPayload {
    #[validate(email(message = "Correo electrónico inválido"))]
    pub email: String,
    #[validate(length(min = 1, message = "La contraseña es obligatoria."))]
    pub password: String,
}

#[derive(Deserialize, Clone, Validate)]
pub struct UpdatePassword {
    #[validate(length(min = 1, message = "La contraseña actual es obligatoria."))]
    pub current_password: String,
    #[validate(custom(function = "validate_password"))]
    pub new_password: String,
}

#[derive(Deserialize, Clone, Validate)]
pub struct ClientMagicTokenLogin {
    #[validate(length(min = 1, message = "Token inválido"))]
    pub magic_token: String,
}
