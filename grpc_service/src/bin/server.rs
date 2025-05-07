use grpc_service::{Error, config::get_config, server::run_server};
use inference_core::modelpool::ModelPool;
use tracing::error;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

fn main() -> Result<(), Error> {
    let config = get_config().expect("failed to build config");
    let log_level = config.log_level.clone().as_str();

    tracing_subscriber::registry()
        .with(JsonStorageLayer)
        .with(
            BunyanFormattingLayer::new("grpc-service".into(), std::io::stdout)
                .skip_fields(vec!["file", "line", "target"].into_iter())
                .unwrap(),
        )
        .with(EnvFilter::try_from_default_env().unwrap_or(log_level.into()))
        .init();

    let model_pool = ModelPool::spawn(1).unwrap();

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async move {
        if let Err(e) = run_server(config, model_pool).await {
            error!(error=%e, "server error");
        };
    });

    Ok(())
}
