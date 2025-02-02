use std::convert::Infallible;
use hyper::StatusCode;
use warp::Reply;
use crate::errors::GatewayError;

pub async fn handle_rejection(err: warp::Rejection) -> Result<impl warp::Reply, Infallible> {
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