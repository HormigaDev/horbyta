use crate::dtos::query::SearchQuery;
use crate::error::AppError;
use axum::{
    Json,
    extract::{FromRequest, FromRequestParts, Query, Request},
    http::request::Parts,
};
use serde::de::DeserializeOwned;
use validator::{Validate, ValidationError, ValidationErrors, ValidationErrorsKind};

fn normalize_message(raw: &str) -> String {
    raw.replace('.', " ")
        .replace(',', " ")
        .replace(':', " ")
        .replace(';', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn format_field_label(path: &str) -> String {
    if path.is_empty() || path == "__all__" {
        return "request".to_string();
    }

    let compact = path
        .replace('.', " ")
        .replace('[', " ")
        .replace(']', " ")
        .replace('_', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    if compact.is_empty() || compact == "all" {
        "request".to_string()
    } else {
        compact
    }
}

fn param_as_string(error: &ValidationError, key: &str) -> Option<String> {
    let value = error.params.get(key)?;

    if let Some(raw) = value.as_str() {
        return Some(raw.to_string());
    }

    if let Some(raw) = value.as_i64() {
        return Some(raw.to_string());
    }

    if let Some(raw) = value.as_u64() {
        return Some(raw.to_string());
    }

    if let Some(raw) = value.as_f64() {
        if raw.fract() == 0.0 {
            return Some(format!("{raw:.0}"));
        }
        return Some(raw.to_string());
    }

    Some(value.to_string())
}

fn format_length_message(field: &str, error: &ValidationError) -> String {
    let min = param_as_string(error, "min");
    let max = param_as_string(error, "max");
    let unit = match error.params.get("value") {
        Some(value) if value.is_array() => "items",
        _ => "characters",
    };

    match (min, max) {
        (Some(min), Some(max)) => format!("{field} must be between {min} and {max} {unit}"),
        (Some(min), None) => format!("{field} must be at least {min} {unit}"),
        (None, Some(max)) => format!("{field} must be at most {max} {unit}"),
        (None, None) => format!("{field} has an invalid length"),
    }
}

fn format_range_message(field: &str, error: &ValidationError) -> String {
    let min = param_as_string(error, "min");
    let max = param_as_string(error, "max");

    match (min, max) {
        (Some(min), Some(max)) => format!("{field} must be between {min} and {max}"),
        (Some(min), None) => format!("{field} must be at least {min}"),
        (None, Some(max)) => format!("{field} must be at most {max}"),
        (None, None) => format!("{field} is out of range"),
    }
}

fn format_custom_message(field: &str, code: &str) -> Option<String> {
    let message = match code {
        "must_be_numeric" => format!("{field} must contain only digits"),
        "password_min_length" => "password must be at least 8 characters".to_string(),
        "password_requires_lowercase" => "password must include at least one lowercase letter".to_string(),
        "password_requires_uppercase" => "password must include at least one uppercase letter".to_string(),
        "password_requires_number" => "password must include at least one number".to_string(),
        "password_requires_symbol" => "password must include at least one symbol".to_string(),
        "admin_managed_password_policy" => {
            "password must be at least 8 characters and include uppercase, lowercase, and number".to_string()
        }
        "invalid_decimal_range" => format!("{field} must be greater than or equal to 0"),
        "invalid_signature_ip" => format!("{field} must be a valid ip address"),
        "invalid_timezone" => format!("{field} must be a valid timezone"),
        "invalid_number_separator" => {
            format!("{field} must use comma or dot as numeric separators")
        }
        "number_separators_must_differ" => "thousand separator and decimal separator must be different".to_string(),
        "invalid_decimal_quantity" => "decimal quantity must be between 1 and 4".to_string(),
        "password_mismatch" => "new password and confirm password must match".to_string(),
        "new_password_must_change" => "new password must be different from current password".to_string(),
        "empty_preferences_patch" => "request must include at least one preference field".to_string(),
        "empty_notifications_patch" => "request must include at least one notification preference field".to_string(),
        "empty_admin_user_patch" => "request must include at least one user field".to_string(),
        "non_negative_decimal" => "decimal must be greater than or equal to 0".to_string(),
        _ => return None,
    };

    Some(message)
}

fn format_field_error(path: &str, error: &ValidationError) -> String {
    if let Some(message) = error.message.as_ref() {
        return normalize_message(message);
    }

    let field = format_field_label(path);
    let code = error.code.as_ref();

    let message = match code {
        "required" => format!("{field} is required"),
        "email" => format!("{field} must be a valid email address"),
        "length" => format_length_message(&field, error),
        "range" => format_range_message(&field, error),
        "must_match" => format!("{field} does not match the expected pattern"),
        "contains" => format!("{field} contains invalid characters"),
        _ => format_custom_message(&field, code).unwrap_or_else(|| format!("{field} is invalid")),
    };

    normalize_message(&message)
}

fn collect_error_messages(errors: &ValidationErrors, current_path: &str, output: &mut Vec<(String, String)>) {
    let mut entries: Vec<_> = errors.errors().iter().collect();
    entries.sort_by(|(left, _), (right, _)| left.cmp(right));

    for (field, kind) in entries {
        let path = if current_path.is_empty() {
            field.to_string()
        } else {
            format!("{current_path}.{field}")
        };

        match kind {
            ValidationErrorsKind::Field(field_errors) => {
                for field_error in field_errors {
                    output.push((path.clone(), format_field_error(&path, field_error)));
                }
            }
            ValidationErrorsKind::Struct(nested) => collect_error_messages(nested, &path, output),
            ValidationErrorsKind::List(list_errors) => {
                let mut list_entries: Vec<_> = list_errors.iter().collect();
                list_entries.sort_by_key(|(index, _)| *index);

                for (index, nested) in list_entries {
                    let indexed_path = format!("{path}[{index}]");
                    collect_error_messages(nested, &indexed_path, output);
                }
            }
        }
    }
}

pub fn first_validation_error_message(errors: &ValidationErrors) -> String {
    let mut messages = Vec::new();
    collect_error_messages(errors, "", &mut messages);
    messages.sort_by(|(left, _), (right, _)| left.cmp(right));

    messages
        .into_iter()
        .map(|(_, message)| message)
        .next()
        .unwrap_or_else(|| "invalid request payload".to_string())
}

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
            .map_err(|_| AppError::Unprocessable("invalid json payload".to_string()))?;

        value
            .validate()
            .map_err(|errors| AppError::BadRequest(first_validation_error_message(&errors)))?;

        Ok(ValidatedJson(value))
    }
}

pub struct ValidatedSearchQuery<T>(pub T);

pub type ValidatedQueryPayload<T> = ValidatedSearchQuery<T>;

impl<S, T> FromRequestParts<S> for ValidatedSearchQuery<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Query(search_query) = Query::<SearchQuery>::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::BadRequest("invalid search query parameters".to_string()))?;

        let value: T = serde_json::from_str(&search_query.q)
            .map_err(|_| AppError::BadRequest("invalid search query payload".to_string()))?;

        value
            .validate()
            .map_err(|errors| AppError::BadRequest(first_validation_error_message(&errors)))?;

        Ok(ValidatedSearchQuery(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use serde::Deserialize;

    #[derive(Debug, Deserialize, Validate)]
    struct JsonTestDto {
        #[validate(length(min = 1, message = "Name: Must, Be.Valid;"))]
        name: String,
    }

    #[derive(Debug, Deserialize, Validate)]
    struct ChildDto {
        #[validate(length(min = 1))]
        name: String,
    }

    #[derive(Debug, Deserialize, Validate)]
    struct ParentDto {
        #[validate(nested)]
        child: ChildDto,
    }

    #[derive(Debug, Deserialize, Validate)]
    struct QueryTestDto {
        #[validate(range(min = 1))]
        page: u64,
    }

    fn request_with_body(body: &'static str) -> Request<Body> {
        Request::builder()
            .header("content-type", "application/json")
            .body(Body::from(body))
            .expect("valid request")
    }

    #[tokio::test]
    async fn validated_json_invalid_json_returns_unprocessable_entity() {
        let result = ValidatedJson::<JsonTestDto>::from_request(request_with_body("{"), &()).await;

        assert!(matches!(
            result,
            Err(AppError::Unprocessable(message)) if message == "invalid json payload"
        ));
    }

    #[tokio::test]
    async fn validated_json_valid_json_with_invalid_dto_returns_bad_request() {
        let result = ValidatedJson::<JsonTestDto>::from_request(request_with_body(r#"{"name":""}"#), &()).await;

        assert!(matches!(
            result,
            Err(AppError::BadRequest(message)) if message == "name must be valid"
        ));
    }

    #[tokio::test]
    async fn validation_message_is_normalized() {
        let errors = JsonTestDto { name: "".to_string() }
            .validate()
            .expect_err("dto should be invalid");

        assert_eq!(first_validation_error_message(&errors), "name must be valid");
    }

    #[tokio::test]
    async fn nested_validation_uses_nested_field_path() {
        let result = ValidatedJson::<ParentDto>::from_request(request_with_body(r#"{"child":{"name":""}}"#), &()).await;

        assert!(matches!(
            result,
            Err(AppError::BadRequest(message)) if message == "child name must be at least 1 characters"
        ));
    }

    #[tokio::test]
    async fn validated_query_payload_invalid_payload_returns_bad_request() {
        let request = Request::builder()
            .uri("/?q=not-json")
            .body(Body::empty())
            .expect("valid request");
        let (mut parts, _) = request.into_parts();

        let result = ValidatedQueryPayload::<QueryTestDto>::from_request_parts(&mut parts, &()).await;

        assert!(matches!(
            result,
            Err(AppError::BadRequest(message)) if message == "invalid search query payload"
        ));
    }

    #[tokio::test]
    async fn validated_query_payload_valid_payload_with_invalid_dto_returns_bad_request() {
        let request = Request::builder()
            .uri("/?q=%7B%22page%22%3A0%7D")
            .body(Body::empty())
            .expect("valid request");
        let (mut parts, _) = request.into_parts();

        let result = ValidatedQueryPayload::<QueryTestDto>::from_request_parts(&mut parts, &()).await;

        assert!(matches!(
            result,
            Err(AppError::BadRequest(message)) if message == "page must be at least 1"
        ));
    }
}
