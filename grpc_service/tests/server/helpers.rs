use grpc_service::{
    ModelSpec,
    config::{DatabaseConfig, get_config},
    inferencer_client::InferencerClient,
    inferencer_server::InferencerServer,
    server::ModelServer,
};
use inference_core::modelpool::ModelPool;
use sqlx::{Connection, Executor, PgConnection, PgPool, postgres::PgConnectOptions};
use std::{env, sync::LazyLock, time::Duration};
use tokio::{net::TcpListener, sync::oneshot, time};
use tokio_stream::StreamExt;
use tonic::{
    Request,
    transport::{Channel, Server},
};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

// tracing
static TRACING: LazyLock<()> = LazyLock::new(|| {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().json())
        .with(EnvFilter::try_from_default_env().unwrap_or("info".into()))
        .init();
});

pub struct TestServer {
    client: InferencerClient<Channel>,
    tx: Option<oneshot::Sender<()>>,
}

impl TestServer {
    pub async fn new(addr: String, tx: oneshot::Sender<()>) -> Self {
        // BAD ? sleep to allow server spawn (could loop)
        time::sleep(Duration::from_millis(10)).await;
        let client = InferencerClient::connect(format!("http://{}", addr))
            .await
            .unwrap();
        Self {
            client,
            tx: Some(tx),
        }
    }

    pub async fn add_models_to_registry(&mut self, models: Vec<ModelSpec>) -> u64 {
        self.client
            .add_models(tokio_stream::iter(models))
            .await
            .unwrap()
            .into_inner()
    }

    pub async fn get_registry_models(&mut self) -> Vec<ModelSpec> {
        let stream = self
            .client
            .list_models(Request::new(()))
            .await
            .expect("failed to get stream")
            .into_inner();

        stream
            .filter_map(|i| i.ok())
            .collect::<Vec<ModelSpec>>()
            .await
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        if let Some(tx) = self.tx.take() {
            // send kill signal to server
            tx.send(()).ok();
        }
    }
}

/// creates new db, connects, and updates config
pub async fn create_test_db(config: &mut DatabaseConfig) -> sqlx::Result<PgPool> {
    let mut opts = PgConnectOptions::new()
        .host(&config.host)
        .port(config.port)
        .username(&config.user_name)
        .password(&config.password);

    let db_name = Uuid::new_v4().to_string();

    // create db
    let mut conn = PgConnection::connect_with(&opts).await?;
    conn.execute(format!(r#"CREATE DATABASE "{}";"#, &db_name).as_str())
        .await?;

    // update config
    opts = opts.database(&db_name);
    config.db_name = db_name;

    let pool = PgPool::connect_with(opts).await?;

    // apply migrations
    sqlx::migrate!("../migrations").run(&pool).await?;

    Ok(pool)
}

// NOTE: not currently working :((
pub async fn delete_test_db(config: &DatabaseConfig) -> sqlx::Result<()> {
    let pool = config.create_pool();
    //drop db
    sqlx::query(format!(r#"DROP DATABASE "{}";"#, &config.db_name).as_str())
        .execute(&pool)
        .await?;
    Ok(())
}

fn set_env_vars() {
    // wokrdir in tests different than main crate
    let dir = env::current_dir().unwrap();

    unsafe {
        env::set_var("CONFIG_FILE", dir.join("config/local.yaml"));
    }
}

pub async fn spawn_server() -> TestServer {
    // set this once
    if env::var("TEST_LOG").is_ok() {
        LazyLock::force(&TRACING);
    }

    set_env_vars();

    let mut config = get_config().unwrap();
    config.db.db_name = Uuid::new_v4().to_string();

    // lazy connect so hopefull we can change db_name here
    let pgpool = create_test_db(&mut config.db).await.unwrap();
    let model_pool = ModelPool::spawn(0).unwrap();
    let model_server = ModelServer::new(pgpool, model_pool).unwrap();

    // set listener on random free port
    let listener = TcpListener::bind("[::1]:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    // cleanup dbs on test termination
    let (tx, rx) = oneshot::channel();
    let shutdown = async move {
        let _ = rx.await;
        delete_test_db(&config.db).await.unwrap();
    };

    // spawn grpc server
    tokio::spawn(async move {
        Server::builder()
            .add_service(InferencerServer::new(model_server))
            .serve_with_incoming_shutdown(
                tokio_stream::wrappers::TcpListenerStream::new(listener),
                shutdown,
            )
            .await
            .unwrap();
    });

    TestServer::new(addr.to_string(), tx).await
}
