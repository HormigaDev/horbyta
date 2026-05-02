use axum::{
    body::to_bytes,
    extract::{FromRequest, Multipart, Request},
    http::header::CONTENT_TYPE,
};
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::{
    error::AppError, middlewares::validator::first_validation_error_message,
    services::attachments::MAX_ATTACHMENT_FILE_BYTES, state::AppState,
};

const MAX_TOTAL_UPLOAD_BYTES: usize = 50 * 1024 * 1024;
const MAX_JSON_BODY_BYTES: usize = 2 * 1024 * 1024;
const PAYLOAD_FIELD_NAME: &str = "payload";
const MARKDOWN_FILE_FIELD_PREFIX: &str = "markdown_file:";

#[derive(Debug)]
pub struct MarkdownUploadedFile {
    pub local_id: String,
    pub original_name: String,
    pub mime_type: Option<String>,
    pub bytes: Vec<u8>,
}

pub fn is_multipart_request(request: &Request) -> bool {
    request
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(|content_type| content_type.starts_with("multipart/form-data"))
        .unwrap_or(false)
}

pub async fn parse_payload_request<T>(
    state: &AppState,
    request: Request,
    missing_payload_message: &'static str,
) -> Result<(T, Vec<MarkdownUploadedFile>), AppError>
where
    T: DeserializeOwned + Validate,
{
    if !is_multipart_request(&request) {
        let bytes = to_bytes(request.into_body(), MAX_JSON_BODY_BYTES)
            .await
            .map_err(|_| AppError::Unprocessable("invalid json payload".to_string()))?;

        let payload: T =
            serde_json::from_slice(&bytes).map_err(|_| AppError::Unprocessable("invalid json payload".to_string()))?;

        payload
            .validate()
            .map_err(|errors| AppError::BadRequest(first_validation_error_message(&errors)))?;

        return Ok((payload, Vec::new()));
    }

    let mut multipart = Multipart::from_request(request, state)
        .await
        .map_err(|_| AppError::Unprocessable("invalid multipart payload".to_string()))?;

    let mut payload: Option<T> = None;
    let mut files = Vec::new();
    let mut total_upload_bytes: usize = 0;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| AppError::BadRequest("failed to process multipart form data".to_string()))?
    {
        let field_name = field.name().unwrap_or_default().to_string();

        if field_name == PAYLOAD_FIELD_NAME {
            if payload.is_some() {
                return Err(AppError::BadRequest(
                    "only one payload field is allowed in multipart request".to_string(),
                ));
            }

            let raw_payload = field
                .text()
                .await
                .map_err(|_| AppError::BadRequest("failed to read payload field".to_string()))?;

            let parsed_payload: T = serde_json::from_str(&raw_payload)
                .map_err(|_| AppError::BadRequest("payload field must contain valid json".to_string()))?;

            parsed_payload
                .validate()
                .map_err(|errors| AppError::BadRequest(first_validation_error_message(&errors)))?;

            payload = Some(parsed_payload);
            continue;
        }

        if let Some(local_id) = field_name.strip_prefix(MARKDOWN_FILE_FIELD_PREFIX) {
            if local_id.trim().is_empty() {
                return Err(AppError::BadRequest(
                    "markdown file field must include a local id".to_string(),
                ));
            }

            let original_name = field.file_name().unwrap_or("archivo").to_string();
            let mime_type = field.content_type().map(|value| value.to_string());
            let bytes = field
                .bytes()
                .await
                .map_err(|_| AppError::BadRequest("failed to read one of the uploaded files".to_string()))?;

            if bytes.len() > MAX_ATTACHMENT_FILE_BYTES as usize {
                return Err(AppError::BadRequest("each file cannot exceed 50mb".to_string()));
            }

            total_upload_bytes = total_upload_bytes.checked_add(bytes.len()).ok_or_else(|| {
                AppError::BadRequest("total uploaded file size exceeds the allowed limit".to_string())
            })?;

            if total_upload_bytes > MAX_TOTAL_UPLOAD_BYTES {
                return Err(AppError::BadRequest(
                    "total uploaded file size must not exceed 50mb".to_string(),
                ));
            }

            files.push(MarkdownUploadedFile {
                local_id: local_id.to_string(),
                original_name,
                mime_type,
                bytes: bytes.to_vec(),
            });
        }
    }

    let payload = payload.ok_or_else(|| AppError::BadRequest(missing_payload_message.to_string()))?;

    Ok((payload, files))
}

pub fn replace_markdown_attachment_urls(content: Option<String>, replacements: &[(String, String)]) -> Option<String> {
    let mut value = content?;

    for (local_id, url) in replacements {
        value = value.replace(&format!("attachment://local/{}", local_id), url);
    }

    Some(value)
}

pub fn replace_markdown_attachment_urls_required(content: String, replacements: &[(String, String)]) -> String {
    replace_markdown_attachment_urls(Some(content), replacements).unwrap_or_default()
}
