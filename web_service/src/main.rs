use web_service::run_app;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    run_app().await?;
    Ok(())
}
