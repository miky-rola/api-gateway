use bytes::Bytes;
use hyper::{Body, Client, Request, Response, Method, HeaderMap};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tokio::time::timeout;
use warp::{Filter, http::Uri};
use api_gateway::{
    AppState,
    GatewayError,
    config::{BACKEND_BASE, REQUEST_TIMEOUT_SECS, STRIP_PATH_PREFIX},
    services::{
        check_rate_limit, 
        get_cached_response, 
        cache_response, 
        is_authenticated
    },
    middleware::add_cors_headers,
    handlers::handle_rejection,
};
use std::convert::Infallible;

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