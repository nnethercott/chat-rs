use tracing_subscriber::{self, EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
use web_service::{config::get_config, server::App};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = get_config().expect("failed to build config");
    // let log_level = config.log_level.clone().as_str();

    // more options from docs  where `with_span_list` indicated
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .json()
                .with_level(true)
                .with_span_list(false), // noise
        )
        .with(EnvFilter::try_from_default_env().unwrap_or("info".into()))
        .init();

    App::new(config)?.run().await?;
    Ok(())
}
