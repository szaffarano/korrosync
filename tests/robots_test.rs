mod common;

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use common::{UnauthenticatedRequestBuilder, spawn_app};
use tower::ServiceExt;

#[tokio::test]
async fn robots_txt_returns_correct_content() {
    let app = spawn_app();

    let response = app
        .oneshot(UnauthenticatedRequestBuilder::get("/robots.txt").build())
        .await
        .expect("Failed to send request");

    assert_eq!(StatusCode::OK, response.status());

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read response body");
    let body_str = String::from_utf8(body.to_vec()).expect("Invalid UTF-8");

    assert_eq!(body_str, "User-agent: *\nDisallow: /");
}

#[tokio::test]
async fn robots_txt_fails_with_invalid_http_methods() {
    let app = spawn_app();

    let methods = [
        Method::POST,
        Method::PUT,
        Method::DELETE,
        Method::PATCH,
        Method::OPTIONS,
        Method::TRACE,
    ];

    for method in methods {
        let req = Request::builder()
            .uri("/robots.txt")
            .method(method.clone())
            .body(Body::empty())
            .unwrap();

        let response = app
            .clone()
            .oneshot(req)
            .await
            .expect("Failed to send request");

        assert_eq!(
            StatusCode::METHOD_NOT_ALLOWED,
            response.status(),
            "Method {method:?} should return 405"
        );
    }
}
