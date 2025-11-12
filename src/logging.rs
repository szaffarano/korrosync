use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt as _};

pub fn init_logging() {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
        }))
        .with(tracing_subscriber::fmt::layer())
        .init();
}
