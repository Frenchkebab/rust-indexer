use anyhow::Result;
use diesel::Connection;
use diesel::SqliteConnection;
use diesel_migrations::MigrationHarness;
use tracing::{Level, info};
use tracing_subscriber::{EnvFilter, fmt};

pub mod config;
pub mod indexer;
pub mod schema;
pub mod types;
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

    // Create Alloy provider for RPC access
    let mut provider = indexer::AlloyProvider {
        url: config.rpc_url.parse()?,
        token_address: config.token_address,
    };

    // Fetch chain_id from RPC and validate against config
    use indexer::LogsProvider;
    let rpc_chain_id = provider
        .chain_id()
        .map_err(|e| anyhow::anyhow!("Failed to get chain ID: {}", e))?;
    if rpc_chain_id != config.chain_id {
        return Err(anyhow::anyhow!(
            "Chain ID mismatch: RPC returned {} but config has {}",
            rpc_chain_id,
            config.chain_id
        ));
    }
    info!("Chain ID verified: {} (matches RPC)", rpc_chain_id);

    // Set start block if not already set
    let is_start_set = indexer::start_from(&mut conn, config.chain_id, config.start_block)?;
    if is_start_set {
        info!("Start block set to {}", config.start_block);
    }

    // Run event loop (blocks until interrupted)
    indexer::event_loop(&mut conn, config.chain_id, provider, 100)?;

    Ok(())
}
