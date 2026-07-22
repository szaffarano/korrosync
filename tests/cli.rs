use assert_cmd::cargo::cargo_bin_cmd;
use korrosync::config::Config;
use tempfile::NamedTempFile;
use tokio_retry2::{Retry, RetryError, strategy::FixedInterval};

#[tokio::test]
#[serial_test::serial]
async fn main_should_start_server() {
    let path = NamedTempFile::new().expect("Creating temp file");
    let mut cfg = Config::from_env();
    cfg.db.path = path.path().to_string_lossy().to_string();
    let app = tokio::spawn(korrosync::run_server(cfg));
    let asserter = assert_server();
    tokio::select! {
        _ = app =>
            panic!("Server task exited unexpectedly")
        ,
        _ = asserter => { },
    }
}

#[tokio::test]
#[serial_test::serial]
async fn cli_should_start_server() {
    let path = NamedTempFile::new().expect("Creating temp file");
    let cmd = cargo_bin_cmd!("korrosync");
    // _cmd is needed to kill the background process on drop
    temp_env::async_with_vars(
        [(
            "KORROSYNC_DB_PATH",
            Some(path.path().to_string_lossy().to_string()),
        )],
        async {
            let _cmd = tokio::process::Command::new(cmd.get_program())
                .arg("serve")
                .kill_on_drop(true)
                .spawn();
            assert_server().await;
        },
    )
    .await;
}

#[tokio::test]
#[serial_test::serial]
async fn run_server_rejects_invalid_bind_address() {
    let path = NamedTempFile::new().expect("Creating temp file");
    let mut cfg = Config::from_env();
    cfg.db.path = path.path().to_string_lossy().to_string();
    cfg.server.address = "not-a-valid-address".into();

    let err = korrosync::run_server(cfg)
        .await
        .expect_err("invalid address should fail");
    assert!(
        err.to_string().contains("Error parsing binding address")
            || format!("{err:#}").contains("Error parsing binding address")
    );
}

#[tokio::test]
#[serial_test::serial]
async fn run_server_graceful_shutdown_on_sigterm() {
    let path = NamedTempFile::new().expect("Creating temp file");
    let mut cfg = Config::from_env();
    cfg.db.path = path.path().to_string_lossy().to_string();

    let server = tokio::spawn(async move { korrosync::run_server(cfg).await });
    assert_server().await;

    let pid = std::process::id().to_string();
    let status = std::process::Command::new("kill")
        .args(["-TERM", &pid])
        .status()
        .expect("failed to send SIGTERM");
    assert!(status.success());

    let result = tokio::time::timeout(std::time::Duration::from_secs(10), server)
        .await
        .expect("server did not shut down in time")
        .expect("server task join failed");
    assert!(result.is_ok(), "server returned error: {result:?}");
}

async fn assert_server() {
    let retry_strategy = FixedInterval::from_millis(10).take(3);
    let response = Retry::spawn(retry_strategy, async || {
        let client = reqwest::Client::new();
        client
            .get("http://127.0.0.1:3000/invalid")
            .send()
            .await
            .map_err(|e| {
                println!("Request failed: {e}, retrying...");
                e
            })
            .map_err(RetryError::transient)
    })
    .await;

    match response {
        Err(e) => panic!("Failed to connect to server after retries with error: {e}"),
        Ok(response) => assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND),
    };
}
