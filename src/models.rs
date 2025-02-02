use std::collections::HashMap;
use std::time::SystemTime;
use hyper::{StatusCode, HeaderMap};
use bytes::Bytes;

pub struct CacheEntry {
    pub response_parts: (StatusCode, HeaderMap, Bytes),
    pub expires_at: SystemTime,
}

pub struct RateLimit {
    pub count: u32,
    pub window_start: SystemTime,
}

impl Default for RateLimit {
    fn default() -> Self {
        Self {
            count: 0,
            window_start: SystemTime::now(),
        }
    }
}

pub struct AppState {
    pub cache: HashMap<String, CacheEntry>,
    pub rate_limits: HashMap<String, RateLimit>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            rate_limits: HashMap::new(),
        }
    }
}