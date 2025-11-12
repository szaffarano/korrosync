use std::time::Duration;

use governor::{clock::QuantaInstant, middleware::NoOpMiddleware};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tower_governor::{
    GovernorLayer, governor::GovernorConfigBuilder, key_extractor::PeerIpKeyExtractor,
};

pub fn rate_limiter_layer<RespBody>(
    shutdown_token: CancellationToken,
) -> (
    GovernorLayer<PeerIpKeyExtractor, NoOpMiddleware<QuantaInstant>, RespBody>,
    JoinHandle<()>,
) {
    let governor_conf = GovernorConfigBuilder::default()
        .per_second(2)
        .burst_size(5)
        .finish()
        .unwrap();

    let governor_limiter = governor_conf.limiter().clone();
    let interval = Duration::from_secs(60);

    let cleanup_task = tokio::spawn(async move {
        // separate background task to clean up
        loop {
            tokio::select! {
                _ = shutdown_token.cancelled() => {
                    tracing::info!("Rate limiter cleanup task shutting down");
                    break;
                }
                _ = tokio::time::sleep(interval) => {
                    tracing::info!("rate limiting storage size: {}", governor_limiter.len());
                    governor_limiter.retain_recent();
                }
            }
        }
    });

    (GovernorLayer::new(governor_conf), cleanup_task)
}
