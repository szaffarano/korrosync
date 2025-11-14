mod common;

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use common::{AuthenticatedRequestBuilder, spawn_app};
use tower::ServiceExt;

#[tokio::test]
async fn users_auth_returns_user_info_with_valid_credentials() {
    let app = spawn_app();

    let response = app
        .oneshot(AuthenticatedRequestBuilder::get("/users/auth").build())
        .await
        .expect("Failed to send request");

    assert_eq!(StatusCode::OK, response.status());

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read response body");
    let body_str = String::from_utf8(body.to_vec()).expect("Invalid UTF-8");
    let body_json: serde_json::Value =
        serde_json::from_str(&body_str).expect("Invalid JSON response");

    assert_eq!(body_json["authorized"], "OK");
    assert_eq!(body_json["username"], "test");
    // last_activity might be present or null
    assert!(body_json.get("last_activity").is_some());
}

#[tokio::test]
async fn users_auth_fails_with_missing_auth_user_header() {
    let app = spawn_app();

    let req = Request::builder()
        .uri("/users/auth")
        .method(Method::GET)
        .header("x-auth-key", "test")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.expect("Failed to send request");

    assert_eq!(StatusCode::UNAUTHORIZED, response.status());

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read response body");
    let body_str = String::from_utf8(body.to_vec()).expect("Invalid UTF-8");
    let body_json: serde_json::Value =
        serde_json::from_str(&body_str).expect("Invalid JSON response");

    assert_eq!(body_json["message"], "Missing credentials");
    assert_eq!(body_json["code"], "unauthorized");
}

#[tokio::test]
async fn users_auth_fails_with_missing_auth_key_header() {
    let app = spawn_app();

    let req = Request::builder()
        .uri("/users/auth")
        .method(Method::GET)
        .header("x-auth-user", "test")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.expect("Failed to send request");

    assert_eq!(StatusCode::UNAUTHORIZED, response.status());

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read response body");
    let body_str = String::from_utf8(body.to_vec()).expect("Invalid UTF-8");
    let body_json: serde_json::Value =
        serde_json::from_str(&body_str).expect("Invalid JSON response");

    assert_eq!(body_json["message"], "Missing credentials");
    assert_eq!(body_json["code"], "unauthorized");
}

#[tokio::test]
async fn users_auth_fails_with_invalid_username() {
    let app = spawn_app();

    let response = app
        .oneshot(
            AuthenticatedRequestBuilder::get("/users/auth")
                .credentials("invaliduser", "test")
                .build(),
        )
        .await
        .expect("Failed to send request");

    assert_eq!(StatusCode::UNAUTHORIZED, response.status());

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read response body");
    let body_str = String::from_utf8(body.to_vec()).expect("Invalid UTF-8");
    let body_json: serde_json::Value =
        serde_json::from_str(&body_str).expect("Invalid JSON response");

    assert_eq!(body_json["message"], "Invalid credentials");
    assert_eq!(body_json["code"], "unauthorized");
}

#[tokio::test]
async fn users_auth_fails_with_invalid_password() {
    let app = spawn_app();

    let response = app
        .oneshot(
            AuthenticatedRequestBuilder::get("/users/auth")
                .credentials("test", "wrongpassword")
                .build(),
        )
        .await
        .expect("Failed to send request");

    assert_eq!(StatusCode::UNAUTHORIZED, response.status());

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read response body");
    let body_str = String::from_utf8(body.to_vec()).expect("Invalid UTF-8");
    let body_json: serde_json::Value =
        serde_json::from_str(&body_str).expect("Invalid JSON response");

    assert_eq!(body_json["message"], "Invalid credentials");
    assert_eq!(body_json["code"], "unauthorized");
}

#[tokio::test]
async fn users_auth_fails_with_invalid_http_methods() {
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
            .uri("/users/auth")
            .method(method.clone())
            .header("x-auth-user", "test")
            .header("x-auth-key", "test")
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
