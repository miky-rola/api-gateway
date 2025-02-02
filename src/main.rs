use bytes::Bytes;
use futures::future::join_all;
use hyper::{Body, Client, Request, Response, StatusCode, Method};
use hyper::header::{HeaderName, HeaderValue};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tokio::time::timeout;
use warp::{http::HeaderMap, Filter};
use std::fmt;
use http::Uri;
// |
// 1   + use hyper::Uri;

// Configuration constants
const BACKEND_BASE: &str = "http://localhost:8081";
const RATE_LIMIT_REQUESTS: u32 = 100; // requests per window
const RATE_LIMIT_WINDOW_SECS: u64 = 60; // window size in seconds
const REQUEST_TIMEOUT_SECS: u64 = 30;
const CACHE_DURATION_SECS: u64 = 300; // 5 minutes
const STRIP_PATH_PREFIX: &str = "/api"; // Strip this prefix before forwarding

// Custom error types
#[derive(Debug)]
enum GatewayError {
    InvalidUri(String),
    Http(String),
    RateLimitExceeded,
    Timeout,
    Unauthorized,
}

impl fmt::Display for GatewayError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidUri(e) => write!(f, "Invalid URI: {}", e),
            Self::Http(e) => write!(f, "HTTP Error: {}", e),
            Self::RateLimitExceeded => write!(f, "Rate limit exceeded"),
            Self::Timeout => write!(f, "Request timed out"),
            Self::Unauthorized => write!(f, "Unauthorized"),
        }
    }
}

impl warp::reject::Reject for GatewayError {}

// Cache entry structure
struct CacheEntry {
    response_parts: (StatusCode, HeaderMap, Bytes),
    expires_at: SystemTime,
}

// Rate limiting structure
struct RateLimit {
    count: u32,
    window_start: SystemTime,
}

impl Default for RateLimit {
    fn default() -> Self {
        Self {
            count: 0,
            window_start: SystemTime::now(),
        }
    }
}

// Shared state
struct AppState {
    cache: HashMap<String, CacheEntry>,
    rate_limits: HashMap<String, RateLimit>,
}

impl AppState {
    fn new() -> Self {
        Self {
            cache: HashMap::new(),
            rate_limits: HashMap::new(),
        }
    }
}

lazy_static! {
    static ref VALID_AUTH_TOKENS: HashMap<String, String> = {
        let mut m = HashMap::new();
        m.insert("example-token".to_string(), "example-user".to_string());
        m
    };
}

#[tokio::main]
async fn main() {
    let state = Arc::new(RwLock::new(AppState::new()));
    let state_filter = warp::any().map(move || state.clone());
    let client = Client::new();

    let health_check = warp::path("health")
        .and(warp::get())
        .map(|| "OK");

    let proxy = warp::any()
        .and(warp::method())
        .and(warp::header::headers_cloned())
        .and(warp::path::full())
        .and(warp::query::raw().or_else(|_| async { Ok::<(String,), Infallible>((String::new(),)) }))
        .and(warp::body::bytes())
        .and(state_filter)
        .and_then(move |method: Method,
                       headers: HeaderMap,
                       full_path: warp::path::FullPath,
                       query: String,
                       body: Bytes,
                       state: Arc<RwLock<AppState>>| {
            let client = client.clone();
            async move {
                let start_time = SystemTime::now();

                if !is_authenticated(&headers) {
                    return Err(warp::reject::custom(GatewayError::Unauthorized));
                }

                if !check_rate_limit(&state, &headers).await {
                    return Err(warp::reject::custom(GatewayError::RateLimitExceeded));
                }

                let cache_key = format!("{}{}{}", method, full_path.as_str(), query);
                if method == Method::GET {
                    if let Some(response) = get_cached_response(&state, &cache_key).await {
                        return Ok(response);
                    }
                }

                let mut path = full_path.as_str().to_string();
                if path.starts_with(STRIP_PATH_PREFIX) {
                    path = path[STRIP_PATH_PREFIX.len()..].to_string();
                }
                
                let mut uri_str = format!("{}{}", BACKEND_BASE, path);
                if !query.is_empty() {
                    uri_str.push('?');
                    uri_str.push_str(&query);
                }

                // Fixed URI parsing with explicit type
                let uri: Uri = uri_str.parse().map_err(|e: hyper::http::uri::InvalidUri| {
                    eprintln!("Failed to parse URI {}: {}", uri_str, e);
                    warp::reject::custom(GatewayError::InvalidUri(e.to_string()))
                })?;

                let mut req_builder = Request::builder()
                    .method(method.clone())
                    .uri(uri);

                for (name, value) in headers.iter() {
                    if name.as_str().to_lowercase() != "host" {
                        req_builder = req_builder.header(name, value);
                    }
                }

                let req = req_builder.body(Body::from(body)).map_err(|e| {
                    eprintln!("Error building request: {}", e);
                    warp::reject::custom(GatewayError::Http(e.to_string()))
                })?;

                let response = match timeout(
                    Duration::from_secs(REQUEST_TIMEOUT_SECS),
                    client.request(req)
                ).await {
                    Ok(result) => result.map_err(|e| {
                        eprintln!("Error forwarding request: {}", e);
                        warp::reject::custom(GatewayError::Http(e.to_string()))
                    })?,
                    Err(_) => return Err(warp::reject::custom(GatewayError::Timeout)),
                };

                let (parts, body) = response.into_parts();
                let body_bytes = hyper::body::to_bytes(body).await.map_err(|e| {
                    eprintln!("Error reading response body: {}", e);
                    warp::reject::custom(GatewayError::Http(e.to_string()))
                })?;

                let mut response = Response::builder()
                    .status(parts.status)
                    .body(Body::from(body_bytes.clone())).unwrap();
                
                let headers = response.headers_mut();
                for (name, value) in parts.headers.iter() {
                    headers.insert(name, value.clone());
                }

                add_cors_headers(headers);

                if method == Method::GET {
                    cache_response(
                        &state,
                        &cache_key,
                        (parts.status, parts.headers, body_bytes),
                    ).await;
                }

                if let Ok(duration) = start_time.elapsed() {
                    println!(
                        "{} {} {} {}ms",
                        method,
                        full_path.as_str(),
                        response.status(),
                        duration.as_millis()
                    );
                }

                Ok(response)
            }
        });

    let routes = health_check
        .or(proxy)
        .recover(handle_rejection);

    println!("API Gateway running on http://127.0.0.1:3030");
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

// Helper functions
async fn check_rate_limit(state: &Arc<RwLock<AppState>>, headers: &HeaderMap) -> bool {
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
        .or_insert_with(|| RateLimit {
            count: 1,
            window_start: now,
        });

    rate_limit.count <= RATE_LIMIT_REQUESTS
}

async fn get_cached_response(state: &Arc<RwLock<AppState>>, cache_key: &str) -> Option<Response<Body>> {
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

async fn cache_response(
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

fn is_authenticated(headers: &HeaderMap) -> bool {
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

fn add_cors_headers(headers: &mut HeaderMap) {
    headers.insert(
        HeaderName::from_static("access-control-allow-origin"),
        HeaderValue::from_static("*"),
    );
    headers.insert(
        HeaderName::from_static("access-control-allow-methods"),
        HeaderValue::from_static("GET, POST, PUT, DELETE, OPTIONS"),
    );
    headers.insert(
        HeaderName::from_static("access-control-allow-headers"),
        HeaderValue::from_static("Content-Type, Authorization"),
    );
}

async fn handle_rejection(err: warp::Rejection) -> Result<impl warp::Reply, Infallible> {
    let (code, message) = if err.is_not_found() {
        (StatusCode::NOT_FOUND, "Not Found")
    } else if let Some(e) = err.find::<GatewayError>() {
        match e {
            GatewayError::RateLimitExceeded => (StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded"),
            GatewayError::Timeout => (StatusCode::GATEWAY_TIMEOUT, "Gateway timeout"),
            GatewayError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized"),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
        }
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
    };

    Ok(warp::reply::with_status(message.to_string(), code))
}