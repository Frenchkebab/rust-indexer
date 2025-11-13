use anyhow::Result;
use diesel::Connection;
use diesel::SqliteConnection;
use diesel_migrations::MigrationHarness;
use tracing::{Level, info};
use tracing_subscriber::{EnvFilter, fmt};

pub mod config;
pub mod schema;
pub mod types;
// pub mod indexer;
// pub mod storage;

pub use config::Config;

pub const MIGRATIONS: diesel_migrations::EmbeddedMigrations =
    diesel_migrations::embed_migrations!("./migrations");

pub fn init_logging() -> Result<()> {
    // Create environment filter with default "info" level
    let mut filter = EnvFilter::new("info");

    // Override filter if RUST_LOG environment variable is set
    if let Ok(var) = std::env::var("RUST_LOG") {
        // Parse and add custom log level directive (e.g., "debug", "rust_indexer=debug")
        filter = filter.add_directive(var.parse()?);
    }

    // Configure and initialize tracing subscriber
    fmt()
        .with_max_level(Level::INFO) // Set maximum log level
        .with_env_filter(filter) // Apply environment-based filtering
        .init(); // Initialize the global logger

    Ok(())
}

pub async fn run(config: Config) -> Result<()> {
    // Format SQLite connection URL (Diesel requires "sqlite://" prefix)
    let database_url = format!("sqlite://{}", config.db_path);

    // Establish database connection, panic if connection fails
    let mut conn = SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));

    // Apply pending migrations
    info!("Applying pending migrations");
    conn.run_pending_migrations(MIGRATIONS)
        .expect("failed to apply migrations");
    info!("Applied pending migrations");

    info!("Starting indexer...");
    info!("  RPC URL: {}", config.rpc_url);
    info!("  Chain ID: {}", config.chain_id);
    info!("  Start Block: {}", config.start_block);
    info!("  DB Path: {}", config.db_path);
    info!("  Token Address: {:#x}", config.token_address);

    // TODO: Create mpsc channels for pipeline
    // let (fetcher_tx, parser_rx) = tokio::sync::mpsc::channel(100);
    // let (parser_tx, storage_rx) = tokio::sync::mpsc::channel(100);

    // TODO: Spawn storage task
    // let storage_handle = tokio::spawn(async move {
    //     storage::run(storage_rx).await
    // });

    // TODO: Spawn parser task
    // let parser_handle = tokio::spawn(async move {
    //     indexer::parse_events(parser_rx, parser_tx).await
    // });

    // TODO: Run fetcher in main task
    // indexer::fetch_events(config, fetcher_tx).await?;

    // TODO: Wait for all tasks to complete
    // parser_handle.await??;
    // storage_handle.await??;

    info!("Indexing completed!");
    Ok(())
}
