use grpc_service::{
    config::{get_config, DatabaseConfig},
    server::run_server,
};
use tokio_postgres::{Client, NoTls, connect};
use tracing::{debug, error, info};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

// need some function to spawn the connection and return client

async fn connect_to_db(config: &DatabaseConfig) -> Result<Client, tokio_postgres::Error> {
    let db_connection_str = format!(
        "host={} port={} user={} password={}",
        &config.host, config.port, &config.username, &config.password
    );
    let (client, connection) = connect(&db_connection_str, NoTls).await?;

    // spawn connection
    tokio::spawn(async move {
        if let Err(_) = connection.await {
            error!("failed to establish connection with db")
        }
    });

    Ok(client)
}

#[tokio::main]
async fn main() -> Result<(), grpc_service::Error>{
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

    // connect to db
    let pg_client = connect_to_db(&config.db).await?;

    let handle = tokio::spawn(async move {
        info!("starting server...");
        debug!(config=?config.server.addr());

        if let Err(e) = run_server(config, pg_client).await {
            error!(error=%e, "server error");
            // NOTE: i'm tempted to build and unwrap an error here ...
        };
    });

    tokio::select! {
        _ = handle => {},
    }

    Ok(())
}
