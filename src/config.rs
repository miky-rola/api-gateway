use std::collections::HashMap;
use lazy_static::lazy_static;

pub const BACKEND_BASE: &str = "http://localhost:8081";
pub const RATE_LIMIT_REQUESTS: u32 = 100; // requests per window
pub const RATE_LIMIT_WINDOW_SECS: u64 = 60; // window size in seconds
pub const REQUEST_TIMEOUT_SECS: u64 = 30;
pub const CACHE_DURATION_SECS: u64 = 300; // 5 minutes
pub const STRIP_PATH_PREFIX: &str = "/api"; // Strip this prefix before forwarding

lazy_static! {
    pub static ref VALID_AUTH_TOKENS: HashMap<String, String> = {
        let mut m = HashMap::new();
        m.insert("example-token".to_string(), "example-user".to_string());
        m
    };
}