use axum::{
    body::Body,
    extract::State,
    http::{
        HeaderMap, HeaderValue, Method, Request, StatusCode,
        header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, ORIGIN},
    },
    middleware::Next,
    response::Response,
};
use tower_http::cors::{AllowOrigin, CorsLayer};

use crate::env::Config;

pub fn cors_layer(config: &Config) -> CorsLayer {
    let allowed_origin = HeaderValue::from_str(&config.frontend_origin).expect("CRITICAL: Invalid FRONTEND_ORIGIN.");

    CorsLayer::new()
        .allow_origin(AllowOrigin::predicate(move |origin, _| origin == allowed_origin))
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([ACCEPT, AUTHORIZATION, CONTENT_TYPE])
        .allow_credentials(true)
}

pub async fn enforce_origin_policy(
    State(config): State<Config>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    if is_allowed_request_origin(request.headers(), &config) {
        Ok(next.run(request).await)
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}

pub(crate) fn is_allowed_request_origin(headers: &HeaderMap, config: &Config) -> bool {
    let Some(origin) = headers.get(ORIGIN) else {
        // Native apps, including Android, normally do not send a browser Origin header.
        return true;
    };

    origin == config.frontend_origin.as_str()
        || headers
            .get("host")
            .is_some_and(|host| origin_matches_host(origin, host))
}

fn origin_matches_host(origin: &HeaderValue, host: &HeaderValue) -> bool {
    let Ok(origin) = origin.to_str() else {
        return false;
    };
    let Ok(host) = host.to_str() else {
        return false;
    };

    origin
        .split_once("://")
        .is_some_and(|(_, origin_host)| origin_host.eq_ignore_ascii_case(host))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config(frontend_origin: &str) -> Config {
        Config {
            database_url: String::new(),
            jwt_secret: String::new(),
            jwt_secret_client: String::new(),
            app_uri: String::new(),
            frontend_origin: frontend_origin.to_string(),
            is_production: true,
            time_to_exp: std::time::Duration::from_secs(60),
            s3_bucket: None,
            s3_endpoint: None,
            s3_region: None,
            s3_access_key: None,
            s3_secret_key: None,
            upload_dir: None,
            allow_registration: false,
        }
    }

    #[test]
    fn allows_requests_without_origin_for_native_clients() {
        let headers = HeaderMap::new();
        assert!(is_allowed_request_origin(
            &headers,
            &test_config("https://app.example.com")
        ));
    }

    #[test]
    fn allows_configured_frontend_origin() {
        let mut headers = HeaderMap::new();
        headers.insert(ORIGIN, HeaderValue::from_static("https://app.example.com"));

        assert!(is_allowed_request_origin(
            &headers,
            &test_config("https://app.example.com")
        ));
    }

    #[test]
    fn allows_same_origin_requests() {
        let mut headers = HeaderMap::new();
        headers.insert(ORIGIN, HeaderValue::from_static("https://api.example.com"));
        headers.insert("host", HeaderValue::from_static("api.example.com"));

        assert!(is_allowed_request_origin(
            &headers,
            &test_config("https://app.example.com")
        ));
    }

    #[test]
    fn rejects_unknown_web_origins() {
        let mut headers = HeaderMap::new();
        headers.insert(ORIGIN, HeaderValue::from_static("https://evil.example.com"));

        assert!(!is_allowed_request_origin(
            &headers,
            &test_config("https://app.example.com")
        ));
    }
}
