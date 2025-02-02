use hyper::{Client, Request, Body, Method};
use std::time::Duration;
use tokio::time::sleep;
use api_gateway::config::BACKEND_BASE;

#[tokio::test]
async fn test_health_check() {
    let client = Client::new();
    let resp = client
        .get("http://127.0.0.1:3030/health".parse().unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn test_proxy_unauthorized() {
    let client = Client::new();
    let resp = client
        .get("http://127.0.0.1:3030/api/test".parse().unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn test_proxy_with_auth() {
    let client = Client::new();
    let req = Request::builder()
        .method(Method::GET)
        .uri("http://127.0.0.1:3030/api/test")
        .header("Authorization", "Bearer example-token")
        .body(Body::empty())
        .unwrap();

    let resp = client.request(req).await.unwrap();
    assert!(resp.status().is_success() || resp.status().is_server_error());
}

#[tokio::test]
async fn test_rate_limiting() {
    let client = Client::new();
    let req = Request::builder()
        .method(Method::GET)
        .uri("http://127.0.0.1:3030/api/test")
        .header("Authorization", "Bearer example-token")
        .body(Body::empty())
        .unwrap();

    // Send requests until rate limit is hit
    for _ in 0..101 {
        let resp = client.request(req.clone()).await.unwrap();
        if resp.status() == 429 {
            return;
        }
        sleep(Duration::from_millis(50)).await;
    }

    panic!("Rate limit was not triggered");
}