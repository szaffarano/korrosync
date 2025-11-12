use assert_cmd::cargo::cargo_bin_cmd;
use korrosync::config::Config;
use tempfile::NamedTempFile;
use tokio_retry2::{Retry, RetryError, strategy::FixedInterval};

#[tokio::test]
async fn main_should_start_server() {
    let path = NamedTempFile::new().expect("Creating temp file");
    let mut cfg = Config::from_env();
    cfg.db.path = path.path().to_string_lossy().to_string();
    tokio::spawn(korrosync::run_server(cfg));
    assert_server().await;
}

#[tokio::test]
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
                .kill_on_drop(true)
                .spawn();
            assert_server().await;
        },
    )
    .await;
}

async fn assert_server() {
    let retry_strategy = FixedInterval::from_millis(10).take(3);
    let response = Retry::spawn(retry_strategy, async || {
        let client = reqwest::Client::new();
        client
            .get("http://127.0.0.1:3000/invalid")
            .send()
            .await
            .map_err(RetryError::transient)
    })
    .await;

    match response {
        Err(e) => panic!("Failed to connect to server after retries with error: {e}"),
        Ok(response) => assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND),
    };
}
