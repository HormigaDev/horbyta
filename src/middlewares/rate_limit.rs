use std::net::IpAddr;
use std::sync::{
    Arc,
    atomic::{AtomicU32, Ordering},
};

use axum::http::HeaderMap;
use moka::sync::Cache;
use uuid::Uuid;

use crate::error::AppError;

// Per-window limits (window = 1 minute, defined by cache TTL in AppState)
pub const AUTH_RATE_LIMIT: u32 = 5;
pub const CHARTS_RATE_LIMIT: u32 = 60;
pub const UPLOADS_RATE_LIMIT: u32 = 10;
pub const MAGIC_TOKEN_RATE_LIMIT: u32 = 5;
pub const WS_MAX_CONNECTIONS: usize = 2;

/// Extracts the real client IP from Nginx-forwarded headers, falling back to
/// 127.0.0.1 in development (direct connection without a proxy).
pub fn extract_client_ip(headers: &HeaderMap) -> IpAddr {
    headers
        .get("x-real-ip")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.trim().parse().ok())
        .or_else(|| {
            headers
                .get("x-forwarded-for")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.split(',').next().map(str::trim))
                .and_then(|s| s.parse().ok())
        })
        .unwrap_or_else(|| IpAddr::from([127, 0, 0, 1]))
}

/// Fixed-window rate limit check keyed by client IP.
/// Returns `TooManyRequests` when the caller has exceeded `limit` requests in
/// the current 1-minute window (defined by the cache TTL set in AppState).
pub fn check_ip_rate_limit(
    limiter: &Cache<IpAddr, Arc<AtomicU32>>,
    headers: &HeaderMap,
    limit: u32,
) -> Result<(), AppError> {
    let ip = extract_client_ip(headers);
    let counter = limiter.get_with(ip, || Arc::new(AtomicU32::new(0)));
    if counter.fetch_add(1, Ordering::Relaxed) >= limit {
        return Err(AppError::TooManyRequests);
    }
    Ok(())
}

/// Fixed-window rate limit check keyed by authenticated user/client ID.
pub fn check_user_rate_limit(limiter: &Cache<Uuid, Arc<AtomicU32>>, user_id: Uuid, limit: u32) -> Result<(), AppError> {
    let counter = limiter.get_with(user_id, || Arc::new(AtomicU32::new(0)));
    if counter.fetch_add(1, Ordering::Relaxed) >= limit {
        return Err(AppError::TooManyRequests);
    }
    Ok(())
}
