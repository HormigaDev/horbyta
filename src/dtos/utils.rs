use validator::ValidationError;

pub fn is_numeric(s: &str) -> Result<(), ValidationError> {
    if s.chars().all(|c| c.is_ascii_digit()) {
        Ok(())
    } else {
        Err(ValidationError::new("El campo debe ser numérico"))
    }
}
