use tracing_subscriber::{self, EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
use web_service::{config::get_config, run_app};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = get_config().expect("failed to build config");
    // let log_level = config.log_level.clone().as_str();

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().json().with_level(true))
        .with(EnvFilter::try_from_default_env().unwrap_or("info".into()))
        .init();

    run_app(config).await?;
    Ok(())
}
