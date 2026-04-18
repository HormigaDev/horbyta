use sea_orm::prelude::Decimal;
use validator::ValidationError;

pub fn is_numeric(s: &str) -> Result<(), ValidationError> {
    if s.chars().all(|c| c.is_ascii_digit()) {
        Ok(())
    } else {
        Err(ValidationError::new("must_be_numeric"))
    }
}

pub fn validate_password(password: &str) -> Result<(), ValidationError> {
    if password.len() < 8 {
        return Err(ValidationError::new("password_min_length"));
    }
    if !password.chars().any(|c| c.is_lowercase()) {
        return Err(ValidationError::new("password_requires_lowercase"));
    }
    if !password.chars().any(|c| c.is_uppercase()) {
        return Err(ValidationError::new("password_requires_uppercase"));
    }
    if !password.chars().any(|c| c.is_ascii_digit()) {
        return Err(ValidationError::new("password_requires_number"));
    }
    if !password.chars().any(|c| !c.is_alphanumeric()) {
        return Err(ValidationError::new("password_requires_symbol"));
    }

    Ok(())
}

pub fn is_non_negative_decimal(decimal: &Decimal) -> Result<(), ValidationError> {
    if decimal < &Decimal::ZERO {
        return Err(ValidationError::new("non_negative_decimal"));
    }

    Ok(())
}
