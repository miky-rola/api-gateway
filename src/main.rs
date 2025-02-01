use bytes::Bytes;
use hyper::{Body, Client, Request, Response, Uri};
use std::convert::Infallible;
use warp::{http::HeaderMap, Filter};
use std::fmt;

/// Custom error types that implement the Reject trait
#[derive(Debug)]
struct InvalidUriError(String);
impl fmt::Display for InvalidUriError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid URI: {}", self.0)
    }
}
impl warp::reject::Reject for InvalidUriError {}

#[derive(Debug)]
struct HttpError(String);
impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "HTTP Error: {}", self.0)
    }
}
impl warp::reject::Reject for HttpError {}

/// The backend URL to which we forward requests.
const BACKEND_BASE: &str = "http://localhost:8081";

#[tokio::main]
async fn main() {
    // Create a shared Hyper client.
    let client = Client::new();

    // Build a Warp filter that captures all requests.
    let proxy = warp::any()
        .and(warp::method())
        .and(warp::header::headers_cloned())
        .and(warp::path::full())
        .and(warp::query::raw().or_else(|_| async { Ok::<(String,), Infallible>((String::new(),)) }))
        .and(warp::body::bytes())
        .and_then(move |method, headers: HeaderMap, full_path: warp::path::FullPath, query: String, body: Bytes| {
            let client = client.clone();
            async move {
                // Build the URI for the backend.
                let mut uri_str = format!("{}{}", BACKEND_BASE, full_path.as_str());
                if !query.is_empty() {
                    uri_str.push('?');
                    uri_str.push_str(&query);
                }
                let uri: Uri = uri_str.parse::<Uri>().map_err(|e: hyper::http::uri::InvalidUri| {
                    eprintln!("Failed to parse URI {}: {}", uri_str, e);
                    warp::reject::custom(InvalidUriError(e.to_string()))
                })?;

                // Start building the request to forward.
                let mut req_builder = Request::builder().method(method).uri(uri);

                // Forward headers except the host header
                for (name, value) in headers.iter() {
                    if name.as_str().to_lowercase() != "host" {
                        req_builder = req_builder.header(name, value);
                    }
                }

                // Create the new request with the captured body.
                let req = req_builder.body(Body::from(body)).map_err(|e| {
                    eprintln!("Error building request: {}", e);
                    warp::reject::custom(HttpError(e.to_string()))
                })?;

                // Forward the request to the backend.
                let backend_response = client.request(req).await.map_err(|e| {
                    eprintln!("Error forwarding request: {}", e);
                    warp::reject::custom(HttpError(e.to_string()))
                })?;

                // Extract the status, headers, and body from the backend response.
                let status = backend_response.status();
                let backend_headers = backend_response.headers().clone();
                let backend_body_bytes = hyper::body::to_bytes(backend_response.into_body())
                    .await
                    .map_err(|e| {
                        eprintln!("Error reading backend response body: {}", e);
                        warp::reject::custom(HttpError(e.to_string()))
                    })?;

                // Build the response that will be returned to the original client.
                let mut response_builder = Response::builder().status(status);
                for (name, value) in backend_headers.iter() {
                    response_builder = response_builder.header(name, value);
                }
                let response = response_builder
                    .body(Body::from(backend_body_bytes))
                    .map_err(|e| {
                        eprintln!("Error building response: {}", e);
                        warp::reject::custom(HttpError(e.to_string()))
                    })?;

                Ok::<_, warp::Rejection>(response)
            }
        });

    // Start the Warp server on port 3030.
    println!("API Gateway running on http://127.0.0.1:3030");
    warp::serve(proxy).run(([127, 0, 0, 1], 3030)).await;
}