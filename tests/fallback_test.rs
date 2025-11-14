mod common;

use axum::body::{Body, HttpBody};
use axum::http::{Method, Request, StatusCode};
use common::{AuthenticatedRequestBuilder, UnauthenticatedRequestBuilder, spawn_app};
use tower::ServiceExt;

#[tokio::test]
async fn fallback_returns_404_for_invalid_routes() {
    let app = spawn_app();

    let invalid_routes = [
        "/invalid",
        "/does/not/exist",
        "/api/v1/users",
        "/healthcheck-invalid",
        "/users/create/extra",
        "/syncs/progress/doc/extra",
    ];

    for route in invalid_routes {
        let response = app
            .clone()
            .oneshot(AuthenticatedRequestBuilder::get(route).build())
            .await
            .expect("Failed to send request");

        assert_eq!(
            StatusCode::NOT_FOUND,
            response.status(),
            "Route {route} should return 404"
        );

        let body_size = response.into_body().size_hint().exact();
        assert_eq!(Some(0), body_size, "Route {route} should have empty body");
    }
}

#[tokio::test]
async fn fallback_returns_404_for_various_http_methods() {
    let app = spawn_app();

    let methods = [
        Method::GET,
        Method::POST,
        Method::PUT,
        Method::DELETE,
        Method::PATCH,
        Method::OPTIONS,
        Method::HEAD,
        Method::TRACE,
    ];

    for method in methods {
        let req = Request::builder()
            .uri("/nonexistent")
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
            StatusCode::NOT_FOUND,
            response.status(),
            "Method {method:?} on invalid route should return 404"
        );

        let body_size = response.into_body().size_hint().exact();
        assert_eq!(
            Some(0),
            body_size,
            "Method {method:?} should have empty body"
        );
    }
}

#[tokio::test]
async fn fallback_works_without_authentication() {
    let app = spawn_app();

    let response = app
        .oneshot(UnauthenticatedRequestBuilder::get("/nonexistent/route").build())
        .await
        .expect("Failed to send request");

    assert_eq!(StatusCode::NOT_FOUND, response.status());

    let body_size = response.into_body().size_hint().exact();
    assert_eq!(Some(0), body_size);
}

#[tokio::test]
async fn fallback_handles_deeply_nested_paths() {
    let app = spawn_app();

    let response = app
        .oneshot(AuthenticatedRequestBuilder::get("/a/b/c/d/e/f/g/h/i/j/k").build())
        .await
        .expect("Failed to send request");

    assert_eq!(StatusCode::NOT_FOUND, response.status());

    let body_size = response.into_body().size_hint().exact();
    assert_eq!(Some(0), body_size);
}
