//! Regression tests for the CLI/API password consistency bug: accounts created with
//! `korrosync user create`/`user reset-password` must authenticate with the MD5-hashed
//! credential that real kosync clients (KOReader, CrossPoint, etc.) send as `x-auth-key`,
//! not with the raw plain-text password typed by the admin.

mod common;

use std::sync::Arc;

use assert_cmd::cargo::cargo_bin_cmd;
use axum::http::StatusCode;
use common::{AuthenticatedRequestBuilder, UnauthenticatedRequestBuilder};
use korrosync::api::{router::app, state::AppState};
use korrosync::model::md5_hex;
use korrosync::service::db::KorrosyncServiceRedb;
use tempfile::NamedTempFile;
use tower::ServiceExt;

/// Runs `korrosync user create` against `db_path` and asserts it succeeded.
fn cli_create_user(db_path: &str, username: &str, password: &str) {
    cargo_bin_cmd!("korrosync")
        .args([
            "--db-path",
            db_path,
            "user",
            "create",
            "--username",
            username,
            "--password",
            password,
        ])
        .assert()
        .success();
}

#[tokio::test]
async fn cli_created_user_authenticates_with_md5_hashed_password() {
    let db_path = NamedTempFile::new().expect("Creating temp file");
    let db_path = db_path.path().to_string_lossy().to_string();
    let real_password = "correct horse battery staple";

    cli_create_user(&db_path, "alice", real_password);

    let sync = Arc::new(
        KorrosyncServiceRedb::new(&db_path).expect("Failed to open database created by the CLI"),
    );
    let app = app(AppState { sync });

    // A real kosync client never sends the plain-text password: it MD5-hashes it first.
    let response = app
        .clone()
        .oneshot(
            AuthenticatedRequestBuilder::get("/users/auth")
                .credentials("alice", &md5_hex(real_password))
                .build(),
        )
        .await
        .expect("Failed to send request");
    assert_eq!(
        StatusCode::OK,
        response.status(),
        "user created via the CLI must authenticate with the MD5-hashed password a real client sends"
    );

    // The raw plain-text password must NOT work as the x-auth-key, since no real client ever
    // sends it that way.
    let response = app
        .oneshot(
            AuthenticatedRequestBuilder::get("/users/auth")
                .credentials("alice", real_password)
                .build(),
        )
        .await
        .expect("Failed to send request");
    assert_eq!(StatusCode::UNAUTHORIZED, response.status());
}

#[tokio::test]
async fn cli_reset_password_authenticates_with_md5_hashed_password() {
    let db_path = NamedTempFile::new().expect("Creating temp file");
    let db_path = db_path.path().to_string_lossy().to_string();
    let old_password = "old password";
    let new_password = "new password";

    cli_create_user(&db_path, "bob", old_password);

    cargo_bin_cmd!("korrosync")
        .args([
            "--db-path",
            &db_path,
            "user",
            "reset-password",
            "--username",
            "bob",
            "--password",
            new_password,
        ])
        .assert()
        .success();

    let sync = Arc::new(
        KorrosyncServiceRedb::new(&db_path).expect("Failed to open database created by the CLI"),
    );
    let app = app(AppState { sync });

    let response = app
        .oneshot(
            AuthenticatedRequestBuilder::get("/users/auth")
                .credentials("bob", &md5_hex(new_password))
                .build(),
        )
        .await
        .expect("Failed to send request");
    assert_eq!(StatusCode::OK, response.status());
}

#[tokio::test]
async fn cli_created_user_matches_equivalent_api_registration() {
    let real_password = "same real password";

    // CLI path: the admin types the real password directly.
    let cli_db_path = NamedTempFile::new().expect("Creating temp file");
    let cli_db_path = cli_db_path.path().to_string_lossy().to_string();
    cli_create_user(&cli_db_path, "carol", real_password);
    let cli_sync = Arc::new(
        KorrosyncServiceRedb::new(&cli_db_path)
            .expect("Failed to open database created by the CLI"),
    );
    let cli_app = app(AppState { sync: cli_sync });

    // API path: a real client MD5-hashes the password before ever calling /users/create.
    let api_db_path = NamedTempFile::new().expect("Creating temp file");
    let api_sync = Arc::new(
        KorrosyncServiceRedb::new(&api_db_path).expect("Failed to create KorrosyncService"),
    );
    let api_app = app(AppState { sync: api_sync });
    let register_body = serde_json::json!({
        "username": "carol",
        "password": md5_hex(real_password),
    })
    .to_string();
    let response = api_app
        .clone()
        .oneshot(
            UnauthenticatedRequestBuilder::post("/users/create")
                .json_body(&register_body)
                .build(),
        )
        .await
        .expect("Failed to send request");
    assert_eq!(StatusCode::CREATED, response.status());

    // Both accounts must authenticate identically when a real client sends MD5(real_password).
    let key = md5_hex(real_password);

    let cli_response = cli_app
        .oneshot(
            AuthenticatedRequestBuilder::get("/users/auth")
                .credentials("carol", &key)
                .build(),
        )
        .await
        .expect("Failed to send request");
    assert_eq!(StatusCode::OK, cli_response.status());

    let api_response = api_app
        .oneshot(
            AuthenticatedRequestBuilder::get("/users/auth")
                .credentials("carol", &key)
                .build(),
        )
        .await
        .expect("Failed to send request");
    assert_eq!(StatusCode::OK, api_response.status());
}
