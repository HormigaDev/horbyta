pub mod crypto;

use validator::ValidationError;

pub fn validate_password(password: &str) -> Result<(), ValidationError> {
    let (mut has_upper, mut has_lower, mut has_number, mut has_symbol) = (false, false, false, false);

    for char in password.chars() {
        match char {
            _ if char.is_uppercase() => has_upper = true,
            _ if char.is_lowercase() => has_lower = true,
            _ if char.is_numeric() => has_number = true,
            _ if !char.is_alphanumeric() => has_symbol = true,
            _ => {}
        };
    }

    if password.len() >= 8 && has_upper && has_lower && has_number && has_symbol {
        Ok(())
    } else {
        Err(ValidationError::new(
            "La contraseña debe tener al menos 8 caracteres, una mayúscula, una minúscula, un número y un símbolo.",
        ))
    }
}
