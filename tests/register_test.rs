mod common;

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use common::{UnauthenticatedRequestBuilder, spawn_app, spawn_app_empty};
use serde_json::json;
use tower::ServiceExt;

#[tokio::test]
async fn register_creates_new_user_successfully() {
    let app = spawn_app_empty();

    let request_body = json!({
        "username": "newuser",
        "password": "newpassword"
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

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read response body");
    let body_str = String::from_utf8(body.to_vec()).expect("Invalid UTF-8");
    let body_json: serde_json::Value =
        serde_json::from_str(&body_str).expect("Invalid JSON response");

    assert_eq!(body_json["username"], "newuser");
}

#[tokio::test]
async fn register_fails_with_empty_username() {
    let app = spawn_app_empty();

    let request_body = json!({
        "username": "",
        "password": "password"
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

    assert_eq!(StatusCode::BAD_REQUEST, response.status());

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read response body");
    let body_str = String::from_utf8(body.to_vec()).expect("Invalid UTF-8");
    let body_json: serde_json::Value =
        serde_json::from_str(&body_str).expect("Invalid JSON response");

    assert_eq!(body_json["code"], "invalid_input");
    assert_eq!(
        body_json["message"],
        "Invalid input: Username and password cannot be empty"
    );
}

#[tokio::test]
async fn register_fails_with_empty_password() {
    let app = spawn_app_empty();

    let request_body = json!({
        "username": "username",
        "password": ""
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

    assert_eq!(StatusCode::BAD_REQUEST, response.status());

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read response body");
    let body_str = String::from_utf8(body.to_vec()).expect("Invalid UTF-8");
    let body_json: serde_json::Value =
        serde_json::from_str(&body_str).expect("Invalid JSON response");

    assert_eq!(body_json["code"], "invalid_input");
    assert_eq!(
        body_json["message"],
        "Invalid input: Username and password cannot be empty"
    );
}

#[tokio::test]
async fn register_fails_with_duplicate_user() {
    let app = spawn_app(); // Already has "test" user

    let request_body = json!({
        "username": "test",
        "password": "newpassword"
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

    assert_eq!(StatusCode::PAYMENT_REQUIRED, response.status());

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read response body");
    let body_str = String::from_utf8(body.to_vec()).expect("Invalid UTF-8");
    let body_json: serde_json::Value =
        serde_json::from_str(&body_str).expect("Invalid JSON response");

    assert_eq!(body_json["message"], "User 'test' already exists");
    assert_eq!(body_json["code"], "existing_user");
}

#[tokio::test]
async fn register_fails_with_invalid_http_methods() {
    let app = spawn_app_empty();

    let methods = [
        Method::GET,
        Method::PUT,
        Method::DELETE,
        Method::PATCH,
        Method::OPTIONS,
        Method::HEAD,
        Method::TRACE,
    ];

    let request_body = json!({
        "username": "testuser",
        "password": "testpassword"
    })
    .to_string();

    for method in methods {
        let req = Request::builder()
            .uri("/users/create")
            .method(method.clone())
            .header("content-type", "application/json")
            .body(Body::from(request_body.clone()))
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
