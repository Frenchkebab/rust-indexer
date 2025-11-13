use anyhow::Result;
use dotenvy::dotenv;
use rust_indexer::{Config, init_logging, run};
use tracing::error;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    init_logging()?;

    let config = Config::from_env()?;
    run(config).await.inspect_err(|e| error!(?e, "run error"))?;

    Ok(())
}
