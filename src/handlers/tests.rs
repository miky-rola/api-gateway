#[cfg(test)]
mod tests {
    // use super::*; I used this but it didnt work, that's why I've commented it out
    use warp::http::StatusCode;
    // use warp::reject::Rejection;
    use crate::handlers::handle_rejection;
    use crate::GatewayError;
    use warp::Reply;

    #[tokio::test]
    async fn test_handle_not_found_rejection() {
        let rejection = warp::reject::not_found();
        let response = handle_rejection(rejection).await.unwrap();
        assert_eq!(response.into_response().status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_handle_rate_limit_rejection() {
        let rejection = warp::reject::custom(GatewayError::RateLimitExceeded);
        let response = handle_rejection(rejection).await.unwrap();
        assert_eq!(response.into_response().status(), StatusCode::TOO_MANY_REQUESTS);
    }

    #[tokio::test]
    async fn test_handle_timeout_rejection() {
        let rejection = warp::reject::custom(GatewayError::Timeout);
        let response = handle_rejection(rejection).await.unwrap();
        assert_eq!(response.into_response().status(), StatusCode::GATEWAY_TIMEOUT);
    }

    #[tokio::test]
    async fn test_handle_unauthorized_rejection() {
        let rejection = warp::reject::custom(GatewayError::Unauthorized);
        let response = handle_rejection(rejection).await.unwrap();
        assert_eq!(response.into_response().status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_handle_unknown_rejection() {
        let rejection = warp::reject::custom(GatewayError::Http("Unknown error".to_string()));
        let response = handle_rejection(rejection).await.unwrap();
        assert_eq!(response.into_response().status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}