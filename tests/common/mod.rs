#![allow(dead_code)]

use axum::Router;
use axum::body::Body;
use axum::http::{Method, Request};
use korrosync::api::{router::app, state::AppState};
use korrosync::model::User;
use korrosync::sync::service::KorrosyncService;
use tempfile::NamedTempFile;

/// Creates a test application with a single test user (username: "test", password: "test")
pub(crate) fn spawn_app() -> Router {
    let db_path = NamedTempFile::new().expect("Creating temp file");
    let sync = KorrosyncService::new(db_path).expect("Failed to create KorrosyncService");

    sync.add_user(&User::new("test", "test").expect("Error instantiating test user"))
        .expect("Error inserting user");

    app(AppState { sync })
}

/// Creates a test application with multiple users
pub(crate) fn spawn_app_with_users(users: Vec<(&str, &str)>) -> Router {
    let db_path = NamedTempFile::new().expect("Creating temp file");
    let sync = KorrosyncService::new(db_path).expect("Failed to create KorrosyncService");

    for (username, password) in users {
        sync.add_user(&User::new(username, password).expect("Error instantiating test user"))
            .expect("Error inserting user");
    }

    app(AppState { sync })
}

/// Creates a test application without any users
pub(crate) fn spawn_app_empty() -> Router {
    let db_path = NamedTempFile::new().expect("Creating temp file");
    let sync = KorrosyncService::new(db_path).expect("Failed to create KorrosyncService");

    app(AppState { sync })
}

/// Helper to create a User instance for testing
pub(crate) fn create_test_user(username: &str, password: &str) -> User {
    User::new(username, password).expect("Error instantiating test user")
}

/// Builder for authenticated requests
pub(crate) struct AuthenticatedRequestBuilder {
    method: Method,
    uri: String,
    username: String,
    password: String,
    body: Body,
}

impl AuthenticatedRequestBuilder {
    pub(crate) fn new(method: Method, uri: &str) -> Self {
        Self {
            method,
            uri: uri.to_string(),
            username: "test".to_string(),
            password: "test".to_string(),
            body: Body::empty(),
        }
    }

    pub(crate) fn get(uri: &str) -> Self {
        Self::new(Method::GET, uri)
    }

    pub(crate) fn post(uri: &str) -> Self {
        Self::new(Method::POST, uri)
    }

    pub(crate) fn put(uri: &str) -> Self {
        Self::new(Method::PUT, uri)
    }

    pub(crate) fn delete(uri: &str) -> Self {
        Self::new(Method::DELETE, uri)
    }

    pub(crate) fn credentials(mut self, username: &str, password: &str) -> Self {
        self.username = username.to_string();
        self.password = password.to_string();
        self
    }

    pub(crate) fn body(mut self, body: Body) -> Self {
        self.body = body;
        self
    }

    pub(crate) fn json_body(mut self, json: &str) -> Self {
        self.body = Body::from(json.to_string());
        self
    }

    pub(crate) fn build(self) -> Request<Body> {
        Request::builder()
            .method(self.method)
            .uri(self.uri)
            .header("x-auth-user", self.username)
            .header("x-auth-key", self.password)
            .header("content-type", "application/json")
            .body(self.body)
            .unwrap()
    }
}

/// Builder for unauthenticated (public) requests
pub(crate) struct UnauthenticatedRequestBuilder {
    method: Method,
    uri: String,
    body: Body,
}

impl UnauthenticatedRequestBuilder {
    pub(crate) fn new(method: Method, uri: &str) -> Self {
        Self {
            method,
            uri: uri.to_string(),
            body: Body::empty(),
        }
    }

    pub(crate) fn get(uri: &str) -> Self {
        Self::new(Method::GET, uri)
    }

    pub(crate) fn post(uri: &str) -> Self {
        Self::new(Method::POST, uri)
    }

    pub(crate) fn put(uri: &str) -> Self {
        Self::new(Method::PUT, uri)
    }

    pub(crate) fn delete(uri: &str) -> Self {
        Self::new(Method::DELETE, uri)
    }

    pub(crate) fn body(mut self, body: Body) -> Self {
        self.body = body;
        self
    }

    pub(crate) fn json_body(mut self, json: &str) -> Self {
        self.body = Body::from(json.to_string());
        self
    }

    pub(crate) fn build(self) -> Request<Body> {
        Request::builder()
            .method(self.method)
            .uri(self.uri)
            .header("content-type", "application/json")
            .body(self.body)
            .unwrap()
    }
}
