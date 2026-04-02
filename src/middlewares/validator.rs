use crate::dtos::query::SearchQuery;
use crate::error::AppError;
use axum::{
    Json,
    extract::{FromRequest, FromRequestParts, Query, Request},
    http::request::Parts,
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

pub struct ValidatedSearchQuery<T>(pub T);

impl<S, T> FromRequestParts<S> for ValidatedSearchQuery<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Query(search_query) = Query::<SearchQuery>::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::BadRequest("Error al leer los parámetros de consulta.".to_string()))?;

        let value: T = serde_json::from_str(&search_query.q).map_err(|_| {
            AppError::BadRequest("Formato JSON inválido o malformado en la consulta de búsqueda.".to_string())
        })?;

        value.validate().map_err(|err| {
            let message = err
                .field_errors()
                .into_values()
                .flatten()
                .filter_map(|e| e.message.as_ref())
                .next()
                .map(|cow| cow.to_string())
                .unwrap_or_else(|| "Error de validación en los parámetros de búsqueda enviados.".to_string());

            AppError::BadRequest(message)
        })?;

        Ok(ValidatedSearchQuery(value))
    }
}
