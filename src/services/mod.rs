use crate::models::{AppState, CacheEntry};
use crate::config::{RATE_LIMIT_REQUESTS, RATE_LIMIT_WINDOW_SECS, CACHE_DURATION_SECS, VALID_AUTH_TOKENS};
use std::sync::Arc;
use tokio::sync::RwLock;
use hyper::{Response, Body, StatusCode, HeaderMap};
use bytes::Bytes;
use std::time::{SystemTime, Duration};

#[cfg(test)]
mod tests;

pub async fn check_rate_limit(state: &Arc<RwLock<AppState>>, headers: &HeaderMap) -> bool {
    let mut state = state.write().await;
    let client_ip = headers
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");

    let now = SystemTime::now();
    let rate_limit = state.rate_limits.entry(client_ip.to_string())
        .and_modify(|rl| {
            if let Ok(duration) = now.duration_since(rl.window_start) {
                if duration.as_secs() >= RATE_LIMIT_WINDOW_SECS {
                    rl.count = 1;
                    rl.window_start = now;
                } else {
                    rl.count += 1;
                }
            }
        })
        .or_insert_with(|| crate::models::RateLimit {
            count: 1,
            window_start: now,
        });

    rate_limit.count <= RATE_LIMIT_REQUESTS
}

pub async fn get_cached_response(state: &Arc<RwLock<AppState>>, cache_key: &str) -> Option<Response<Body>> {
    let state = state.read().await;
    if let Some(entry) = state.cache.get(cache_key) {
        if SystemTime::now() < entry.expires_at {
            let (status, headers, body) = entry.response_parts.clone();
            let mut response = Response::builder()
                .status(status)
                .body(Body::from(body))
                .unwrap();
            *response.headers_mut() = headers;
            return Some(response);
        }
    }
    None
}

pub async fn cache_response(
    state: &Arc<RwLock<AppState>>,
    cache_key: &str,
    response_parts: (StatusCode, HeaderMap, Bytes),
) {
    let mut state = state.write().await;
    state.cache.insert(
        cache_key.to_string(),
        CacheEntry {
            response_parts,
            expires_at: SystemTime::now() + Duration::from_secs(CACHE_DURATION_SECS),
        },
    );
}

pub fn is_authenticated(headers: &HeaderMap) -> bool {
    if let Some(auth_header) = headers.get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                let token = &auth_str[7..];
                return VALID_AUTH_TOKENS.contains_key(token);
            }
        }
    }
    false
}