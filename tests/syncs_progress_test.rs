mod common;

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use common::{AuthenticatedRequestBuilder, spawn_app};
use serde_json::json;
use tower::ServiceExt;

#[tokio::test]
async fn put_syncs_progress_updates_successfully() {
    let app = spawn_app();

    let request_body = json!({
        "device_id": "device123",
        "device": "MyDevice",
        "document": "test_doc.epub",
        "percentage": 0.75,
        "progress": "Chapter 5"
    })
    .to_string();

    let response = app
        .oneshot(
            AuthenticatedRequestBuilder::put("/syncs/progress")
                .json_body(&request_body)
                .build(),
        )
        .await
        .expect("Failed to send request");

    assert_eq!(StatusCode::OK, response.status());

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read response body");
    let body_str = String::from_utf8(body.to_vec()).expect("Invalid UTF-8");
    let body_json: serde_json::Value =
        serde_json::from_str(&body_str).expect("Invalid JSON response");

    assert_eq!(body_json["document"], "test_doc.epub");
    assert!(body_json.get("timestamp").is_some());
    assert!(body_json["timestamp"].is_number());
}

#[tokio::test]
async fn put_syncs_progress_fails_without_auth() {
    let app = spawn_app();

    let request_body = json!({
        "device_id": "device123",
        "device": "MyDevice",
        "document": "test_doc.epub",
        "percentage": 0.75,
        "progress": "Chapter 5"
    })
    .to_string();

    let req = Request::builder()
        .uri("/syncs/progress")
        .method(Method::PUT)
        .header("content-type", "application/json")
        .body(Body::from(request_body))
        .unwrap();

    let response = app.oneshot(req).await.expect("Failed to send request");

    assert_eq!(StatusCode::UNAUTHORIZED, response.status());

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read response body");
    let body_str = String::from_utf8(body.to_vec()).expect("Invalid UTF-8");
    let body_json: serde_json::Value =
        serde_json::from_str(&body_str).expect("Invalid JSON response");

    assert_eq!(body_json["error"], "Missing credentials");
}

#[tokio::test]
async fn put_syncs_progress_fails_with_invalid_http_methods() {
    let app = spawn_app();

    let methods = [
        Method::GET,
        Method::POST,
        Method::DELETE,
        Method::PATCH,
        Method::OPTIONS,
        Method::HEAD,
        Method::TRACE,
    ];

    let request_body = json!({
        "device_id": "device123",
        "device": "MyDevice",
        "document": "test_doc.epub",
        "percentage": 0.75,
        "progress": "Chapter 5"
    })
    .to_string();

    for method in methods {
        let req = Request::builder()
            .uri("/syncs/progress")
            .method(method.clone())
            .header("x-auth-user", "test")
            .header("x-auth-key", "test")
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

#[tokio::test]
async fn get_syncs_progress_retrieves_document_successfully() {
    let app = spawn_app();

    let request_body = json!({
        "device_id": "device456",
        "device": "TestDevice",
        "document": "my_book.pdf",
        "percentage": 0.5,
        "progress": "Page 100"
    })
    .to_string();

    let _put_response = app
        .clone()
        .oneshot(
            AuthenticatedRequestBuilder::put("/syncs/progress")
                .json_body(&request_body)
                .build(),
        )
        .await
        .expect("Failed to send PUT request");

    let response = app
        .oneshot(AuthenticatedRequestBuilder::get("/syncs/progress/my_book.pdf").build())
        .await
        .expect("Failed to send GET request");

    assert_eq!(StatusCode::OK, response.status());

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read response body");
    let body_str = String::from_utf8(body.to_vec()).expect("Invalid UTF-8");
    let body_json: serde_json::Value =
        serde_json::from_str(&body_str).expect("Invalid JSON response");

    assert_eq!(body_json["device_id"], "device456");
    assert_eq!(body_json["device"], "TestDevice");
    assert_eq!(body_json["document"], "my_book.pdf");
    assert_eq!(body_json["percentage"], 0.5);
    assert_eq!(body_json["progress"], "Page 100");
    assert!(body_json.get("timestamp").is_some());
}

#[tokio::test]
async fn get_syncs_progress_returns_empty_json_for_missing_document() {
    let app = spawn_app();

    let response = app
        .oneshot(AuthenticatedRequestBuilder::get("/syncs/progress/nonexistent_doc.epub").build())
        .await
        .expect("Failed to send request");

    assert_eq!(StatusCode::OK, response.status());

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read response body");
    let body_str = String::from_utf8(body.to_vec()).expect("Invalid UTF-8");
    let body_json: serde_json::Value =
        serde_json::from_str(&body_str).expect("Invalid JSON response");

    // Should return empty JSON object for KOReader compatibility
    assert_eq!(body_json, json!({}));
}

#[tokio::test]
async fn get_syncs_progress_fails_without_auth() {
    let app = spawn_app();

    let req = Request::builder()
        .uri("/syncs/progress/test_doc.epub")
        .method(Method::GET)
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

    assert_eq!(body_json["error"], "Missing credentials");
}

#[tokio::test]
async fn get_syncs_progress_fails_with_invalid_http_methods() {
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
            .uri("/syncs/progress/test_doc.epub")
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
