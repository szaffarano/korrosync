mod common;

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use common::{
    AuthenticatedRequestBuilder, UnauthenticatedRequestBuilder, spawn_app, spawn_app_with_users,
};
use serde_json::json;
use tower::ServiceExt;

// ==================== AUTH MIDDLEWARE TESTS ====================

#[tokio::test]
async fn auth_middleware_accepts_valid_credentials() {
    let app = spawn_app_with_users(vec![("alice", "password123"), ("bob", "secret456")]);

    let response = app
        .clone()
        .oneshot(
            AuthenticatedRequestBuilder::get("/healthcheck")
                .credentials("alice", "password123")
                .build(),
        )
        .await
        .expect("Failed to send request");

    assert_eq!(StatusCode::OK, response.status());

    let response = app
        .oneshot(
            AuthenticatedRequestBuilder::get("/healthcheck")
                .credentials("bob", "secret456")
                .build(),
        )
        .await
        .expect("Failed to send request");

    assert_eq!(StatusCode::OK, response.status());
}

#[tokio::test]
async fn auth_middleware_rejects_missing_x_auth_user() {
    let app = spawn_app();

    let req = Request::builder()
        .uri("/healthcheck")
        .method(Method::GET)
        .header("x-auth-key", "test")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.expect("Failed to send request");

    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read response body");
    let body_str = String::from_utf8(body.to_vec()).expect("Invalid UTF-8");
    let body_json: serde_json::Value =
        serde_json::from_str(&body_str).expect("Invalid JSON response");

    assert_eq!(StatusCode::UNAUTHORIZED, status);
    assert_eq!(body_json["message"], "Missing credentials");
    assert_eq!(body_json["code"], "unauthorized");
}

#[tokio::test]
async fn auth_middleware_rejects_missing_x_auth_key() {
    let app = spawn_app();

    let req = Request::builder()
        .uri("/healthcheck")
        .method(Method::GET)
        .header("x-auth-user", "test")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.expect("Failed to send request");

    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read response body");
    let body_str = String::from_utf8(body.to_vec()).expect("Invalid UTF-8");
    let body_json: serde_json::Value =
        serde_json::from_str(&body_str).expect("Invalid JSON response");

    assert_eq!(StatusCode::UNAUTHORIZED, status);
    assert_eq!(body_json["message"], "Missing credentials");
    assert_eq!(body_json["code"], "unauthorized");
}

#[tokio::test]
async fn auth_middleware_rejects_missing_both_headers() {
    let app = spawn_app();

    let req = Request::builder()
        .uri("/healthcheck")
        .method(Method::GET)
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.expect("Failed to send request");

    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read response body");
    let body_str = String::from_utf8(body.to_vec()).expect("Invalid UTF-8");
    let body_json: serde_json::Value =
        serde_json::from_str(&body_str).expect("Invalid JSON response");

    assert_eq!(StatusCode::UNAUTHORIZED, status);
    assert_eq!(body_json["message"], "Missing credentials");
    assert_eq!(body_json["code"], "unauthorized");
}

#[tokio::test]
async fn auth_middleware_rejects_invalid_username() {
    let app = spawn_app();

    let response = app
        .oneshot(
            AuthenticatedRequestBuilder::get("/healthcheck")
                .credentials("nonexistent", "test")
                .build(),
        )
        .await
        .expect("Failed to send request");

    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read response body");
    let body_str = String::from_utf8(body.to_vec()).expect("Invalid UTF-8");
    let body_json: serde_json::Value =
        serde_json::from_str(&body_str).expect("Invalid JSON response");

    assert_eq!(StatusCode::UNAUTHORIZED, status);
    assert_eq!(body_json["message"], "Invalid credentials");
    assert_eq!(body_json["code"], "unauthorized");
}

#[tokio::test]
async fn auth_middleware_rejects_invalid_password() {
    let app = spawn_app();

    let response = app
        .oneshot(
            AuthenticatedRequestBuilder::get("/healthcheck")
                .credentials("test", "wrongpassword")
                .build(),
        )
        .await
        .expect("Failed to send request");

    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read response body");
    let body_str = String::from_utf8(body.to_vec()).expect("Invalid UTF-8");
    let body_json: serde_json::Value =
        serde_json::from_str(&body_str).expect("Invalid JSON response");

    assert_eq!(StatusCode::UNAUTHORIZED, status);
    assert_eq!(body_json["message"], "Invalid credentials");
    assert_eq!(body_json["code"], "unauthorized");
}

#[tokio::test]
async fn auth_middleware_updates_user_last_activity() {
    let app = spawn_app();

    let response1 = app
        .clone()
        .oneshot(AuthenticatedRequestBuilder::get("/users/auth").build())
        .await
        .expect("Failed to send request");

    assert_eq!(StatusCode::OK, response1.status());

    let body1 = axum::body::to_bytes(response1.into_body(), usize::MAX)
        .await
        .expect("Failed to read response body");
    let body1_str = String::from_utf8(body1.to_vec()).expect("Invalid UTF-8");
    let body1_json: serde_json::Value =
        serde_json::from_str(&body1_str).expect("Invalid JSON response");

    let first_activity = body1_json.get("last_activity");

    // Wait a moment to ensure timestamp changes
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    let response2 = app
        .oneshot(AuthenticatedRequestBuilder::get("/users/auth").build())
        .await
        .expect("Failed to send request");

    assert_eq!(StatusCode::OK, response2.status());

    let body2 = axum::body::to_bytes(response2.into_body(), usize::MAX)
        .await
        .expect("Failed to read response body");
    let body2_str = String::from_utf8(body2.to_vec()).expect("Invalid UTF-8");
    let body2_json: serde_json::Value =
        serde_json::from_str(&body2_str).expect("Invalid JSON response");

    let second_activity = body2_json.get("last_activity");

    assert!(first_activity.is_some());
    assert!(second_activity.is_some());

    if let (Some(first), Some(second)) = (
        first_activity.and_then(|v| v.as_i64()),
        second_activity.and_then(|v| v.as_i64()),
    ) {
        assert!(
            second >= first,
            "Second last_activity ({second}) should be >= first ({first})"
        );
    }
}

#[tokio::test]
async fn auth_middleware_applies_to_all_protected_routes() {
    let app = spawn_app();

    let protected_routes = vec![
        ("/healthcheck", Method::GET),
        ("/users/auth", Method::GET),
        ("/syncs/progress", Method::PUT),
        ("/syncs/progress/test.epub", Method::GET),
    ];

    for (route, method) in protected_routes {
        let req = Request::builder()
            .uri(route)
            .method(method.clone())
            .body(Body::empty())
            .unwrap();

        let response = app
            .clone()
            .oneshot(req)
            .await
            .expect("Failed to send request");

        assert_eq!(
            StatusCode::UNAUTHORIZED,
            response.status(),
            "Protected route {route} with method {method:?} should require auth"
        );
    }
}

// ==================== PUBLIC MIDDLEWARE TESTS ====================

#[tokio::test]
async fn public_middleware_allows_access_without_auth() {
    let app = spawn_app();

    let response = app
        .clone()
        .oneshot(UnauthenticatedRequestBuilder::get("/robots.txt").build())
        .await
        .expect("Failed to send request");

    assert_eq!(StatusCode::OK, response.status());

    let request_body = json!({
        "username": "newuser",
        "password": "newpass"
    })
    .to_string();

    let response = app
        .oneshot(
            UnauthenticatedRequestBuilder::post("/users/create")
                .json_body(&request_body)
                .build(),
        )
        .await
        .expect("Failed to send request");

    assert_eq!(StatusCode::CREATED, response.status());
}

#[tokio::test]
async fn public_middleware_does_not_require_x_auth_headers() {
    let app = spawn_app();

    let public_routes = vec![
        ("/robots.txt", Method::GET),
        ("/users/create", Method::POST),
    ];

    for (route, method) in public_routes {
        let req = Request::builder()
            .uri(route)
            .method(method.clone())
            .body(Body::empty())
            .unwrap();

        let response = app
            .clone()
            .oneshot(req)
            .await
            .expect("Failed to send request");

        assert_ne!(
            StatusCode::UNAUTHORIZED,
            response.status(),
            "Public route {route} with method {method:?} should not require auth"
        );
    }
}

#[tokio::test]
async fn public_routes_work_even_with_invalid_auth_headers() {
    let app = spawn_app();

    let req = Request::builder()
        .uri("/robots.txt")
        .method(Method::GET)
        .header("x-auth-user", "invalid")
        .header("x-auth-key", "invalid")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.expect("Failed to send request");

    assert_eq!(StatusCode::OK, response.status());
}
