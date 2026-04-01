use crate::error::AppError;
use axum::{
    Json,
    extract::{FromRequest, Request},
};
use serde::de::DeserializeOwned;
use validator::Validate;

pub struct ValidatedJson<T>(pub T);

impl<S, T> FromRequest<S> for ValidatedJson<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|_| AppError::Unprocessable("Formato JSON inválido o malformado.".to_string()))?;

        value.validate().map_err(|err| {
            let message = err
                .field_errors()
                .into_values()
                .flatten()
                .filter_map(|e| e.message.as_ref())
                .next()
                .map(|cow| cow.to_string())
                .unwrap_or_else(|| "Error de validación en los campos enviados.".to_string());

            AppError::BadRequest(message)
        })?;

        Ok(ValidatedJson(value))
    }
}
