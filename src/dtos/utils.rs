use validator::ValidationError;

pub fn is_numeric(s: &str) -> Result<(), ValidationError> {
    if s.chars().all(|c| c.is_ascii_digit()) {
        Ok(())
    } else {
        Err(ValidationError::new("El campo debe ser numérico"))
    }
}

pub fn validate_password(password: &str) -> Result<(), ValidationError> {
    if password.len() < 12 {
        return Err(ValidationError::new("Debe tener al menos 12 caracteres"));
    }
    if !password.chars().any(|c| c.is_lowercase()) {
        return Err(ValidationError::new("Falta minúscula"));
    }
    if !password.chars().any(|c| c.is_uppercase()) {
        return Err(ValidationError::new("Falta mayúscula"));
    }
    if !password.chars().any(|c| c.is_ascii_digit()) {
        return Err(ValidationError::new("Falta número"));
    }
    if !password.chars().any(|c| !c.is_alphanumeric()) {
        return Err(ValidationError::new("Falta símbolo"));
    }

    Ok(())
}
