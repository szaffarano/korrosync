mod common;

use axum::body::HttpBody;
use axum::http::{Method, StatusCode};
use tower::ServiceExt;

use crate::common::{AuthenticatedRequestBuilder, spawn_app};

#[tokio::test]
async fn health_check_works() {
    let app = spawn_app();

    let response = app
        .oneshot(AuthenticatedRequestBuilder::get("/healthcheck").build())
        .await
        .expect("Failed to send request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.into_body().size_hint().exact());
}

#[tokio::test]
async fn health_check_fails_with_invalid_verb() {
    let app = spawn_app();

    let methods = [
        Method::PUT,
        Method::POST,
        Method::DELETE,
        Method::PATCH,
        Method::OPTIONS,
        Method::TRACE,
    ];

    for method in methods {
        let response = app
            .clone()
            .oneshot(AuthenticatedRequestBuilder::new(method.clone(), "/healthcheck").build())
            .await
            .expect("Failed to send request");

        assert_eq!(
            StatusCode::METHOD_NOT_ALLOWED,
            response.status(),
            "Method {method:?} should return 405",
        );
        assert_eq!(Some(0), response.into_body().size_hint().exact());
    }
}
