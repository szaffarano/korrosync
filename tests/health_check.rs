use korrosync::api::{router::serve, state::AppState};
use korrosync::sync::{KorrosyncService, User};
use reqwest::{Method, StatusCode};
use tempfile::NamedTempFile;
use tokio::net::TcpListener;

#[tokio::test]
async fn health_check_works() {
    let port = spawn_server().await;

    let client = reqwest::Client::new();

    let response = client
        .get(format!("http://127.0.0.1:{port}/healthcheck"))
        .header("x-auth-user", "test")
        .header("x-auth-key", "test")
        .send()
        .await
        .expect("Failed to send request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn health_check_fails_with_invalid_url() {
    let port = spawn_server().await;

    let client = reqwest::Client::new();

    let response = client
        .get(format!("http://127.0.0.1:{port}/invalid"))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(StatusCode::NOT_FOUND, response.status());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn health_check_fails_with_invalid_verb() {
    let port = spawn_server().await;
    let url = format!("http://127.0.0.1:{port}/healthcheck");
    let client = reqwest::Client::new();

    let methods = [
        Method::PUT,
        Method::DELETE,
        Method::PATCH,
        Method::OPTIONS,
        Method::TRACE,
    ];

    for method in methods {
        let response = client
            .request(method.clone(), &url)
            .send()
            .await
            .expect("Failed to send request");

        assert_eq!(
            StatusCode::UNAUTHORIZED,
            response.status(),
            "Method {method:?} should return 405",
        );
        assert_eq!(Some(31), response.content_length());
    }
}

async fn spawn_server() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind to address");

    let db_path = NamedTempFile::new().expect("Creating temp file");
    let port = listener.local_addr().unwrap().port();
    let state = AppState {
        sync: KorrosyncService::new(db_path).expect("Failed to create KorrosyncService"),
    };

    state
        .sync
        .add_user(User::new("test", "test").expect("Error instantiating test user"))
        .expect("Error inserting user");

    tokio::spawn(async move {
        serve(listener, state)
            .await
            .expect("Failed to start server")
    });

    port
}
