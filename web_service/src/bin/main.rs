use clap::Parser;
use tracing_subscriber::{self, EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
use web_service::{config::Settings, server::App};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Settings::parse();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_level(true)
                // .with_span_list(false), // noise
        )
        .with(EnvFilter::try_from_default_env().unwrap_or("info".into()))
        .init();

    dbg!("{:?}", &config);
    App::new_with_session_store(config).await?.run().await?;
    Ok(())
}
