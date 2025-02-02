#[cfg(test)]
mod tests {
    // use super::*; I used this but it didnt work, that's why I've commented it out
    use crate::AppState;
    use hyper::{HeaderMap, header::AUTHORIZATION};
    use std::time::Duration;
    use tokio::sync::RwLock;
    use std::sync::Arc;
    // use crate::services::check_rate_limit;
    use crate::services::{
        RATE_LIMIT_REQUESTS, 
        RATE_LIMIT_WINDOW_SECS, 
        StatusCode, 
        Bytes, 
        cache_response, 
        get_cached_response, 
        SystemTime, 
        is_authenticated, 
        check_rate_limit
    };
    use crate::RateLimit;
    // use crate::services::SystemTime;
    use crate::CacheEntry;

    #[tokio::test]
    async fn test_rate_limit() {
        let state = Arc::new(RwLock::new(AppState::new()));
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", "127.0.0.1".parse().unwrap());

        // First request should pass
        assert!(check_rate_limit(&state, &headers).await);

        // Add more requests up to the limit
        {
            let mut state = state.write().await;
            let rate_limit = state.rate_limits.get_mut("127.0.0.1").unwrap();
            rate_limit.count = RATE_LIMIT_REQUESTS;
        }

        // Next request should fail
        assert!(!check_rate_limit(&state, &headers).await);
    }

    #[tokio::test]
    async fn test_rate_limit_window_reset() {
        let state = Arc::new(RwLock::new(AppState::new()));
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", "127.0.0.1".parse().unwrap());

        // Add requests at limit
        {
            let mut state = state.write().await;
            state.rate_limits.insert(
                "127.0.0.1".to_string(),
                RateLimit {
                    count: RATE_LIMIT_REQUESTS,
                    window_start: SystemTime::now() - Duration::from_secs(RATE_LIMIT_WINDOW_SECS + 1),
                },
            );
        }

        // Should pass because window has reset
        assert!(check_rate_limit(&state, &headers).await);
    }

    #[tokio::test]
    async fn test_cache_operations() {
        let state = Arc::new(RwLock::new(AppState::new()));
        let cache_key = "test_key";
        let status = StatusCode::OK;
        let headers = HeaderMap::new();
        let body = Bytes::from("test body");

        // Cache a response
        cache_response(
            &state,
            cache_key,
            (status, headers.clone(), body.clone()),
        ).await;

        // Retrieve cached response
        let cached_response = get_cached_response(&state, cache_key).await;
        assert!(cached_response.is_some());

        if let Some(response) = cached_response {
            assert_eq!(response.status(), status);
            let cached_body = hyper::body::to_bytes(response.into_body()).await.unwrap();
            assert_eq!(cached_body, body);
        }
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let state = Arc::new(RwLock::new(AppState::new()));
        let cache_key = "test_key";
        
        // Cache a response with immediate expiration
        {
            let mut state = state.write().await;
            state.cache.insert(
                cache_key.to_string(),
                CacheEntry {
                    response_parts: (
                        StatusCode::OK,
                        HeaderMap::new(),
                        Bytes::from("test"),
                    ),
                    expires_at: SystemTime::now() - Duration::from_secs(1),
                },
            );
        }

        // Should return None for expired cache
        let cached_response = get_cached_response(&state, cache_key).await;
        assert!(cached_response.is_none());
    }

    #[tokio::test]
    async fn test_authentication() {
        let mut headers = HeaderMap::new();
        
        // Test invalid auth header
        headers.insert(AUTHORIZATION, "Invalid".parse().unwrap());
        assert!(!is_authenticated(&headers));

        // Test invalid bearer token
        headers.insert(AUTHORIZATION, "Bearer invalid-token".parse().unwrap());
        assert!(!is_authenticated(&headers));

        // Test valid bearer token
        headers.insert(AUTHORIZATION, "Bearer example-token".parse().unwrap());
        assert!(is_authenticated(&headers));
    }
}