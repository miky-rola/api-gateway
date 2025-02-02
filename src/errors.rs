use std::fmt;

#[derive(Debug)]
pub enum GatewayError {
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