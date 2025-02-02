#[cfg(test)]
mod tests {
    // use super::*; I used this but it didnt work, that's why I've commented it out
    use hyper::HeaderMap;
    use crate::middleware::add_cors_headers;

    #[test]
    fn test_add_cors_headers() {
        let mut headers = HeaderMap::new();
        add_cors_headers(&mut headers);

        assert_eq!(
            headers.get("access-control-allow-origin").unwrap(),
            "*"
        );
        assert_eq!(
            headers.get("access-control-allow-methods").unwrap(),
            "GET, POST, PUT, DELETE, PATCH, OPTIONS"
        );
        assert_eq!(
            headers.get("access-control-allow-headers").unwrap(),
            "Content-Type, Authorization"
        );
    }
}