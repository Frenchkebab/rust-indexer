use crate::schema;
use alloy::primitives::Address;
use alloy::providers::Provider;
use alloy::rpc::types::Filter;
use alloy::rpc::types::eth::Log;
use alloy::transports::http::reqwest::Url;
use diesel::prelude::*;

#[derive(thiserror::Error, Debug)]
pub enum IndexerError {
    #[error("RPC error: {0}")]
    Rpc(String),

    #[error("Database error: {0}")]
    Database(#[from] diesel::result::Error),

    #[error("Chain ID mismatch: RPC returned {rpc} but expected {expected}")]
    ChainIdMismatch { rpc: u64, expected: u64 },

    #[error("Runtime error: {0}")]
    Runtime(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),
}

pub type Result<T> = std::result::Result<T, IndexerError>;

// keccak256 hash of the Transfer event signature
const TRANSFER_EVENT_SIGNATURE: &str =
    "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";

pub trait LogsProvider {
    fn latest_block(&mut self) -> Result<u64>;

    fn chain_id(&mut self) -> Result<u64>;

    fn logs(&self, start_block: u64, end_block: u64) -> Result<impl IntoIterator<Item = Log>>;
}

#[derive(Clone)]
pub struct AlloyProvider {
    pub url: Url,
    pub token_address: Address,
}

impl LogsProvider for AlloyProvider {
    // Fetch the latest block number from the RPC endpoint
    fn latest_block(&mut self) -> Result<u64> {
        // Create tokio runtime for async operations (Diesel is synchronous)
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| IndexerError::Runtime(e))?;

        // Create Alloy HTTP provider connected to the RPC URL
        let provider = alloy::providers::ProviderBuilder::new().connect_http(self.url.clone());
        // Block on the async get_block_number() call and return the result
        // This converts the async operation to a synchronous one
        rt.block_on(provider.get_block_number())
            .map_err(|e| IndexerError::Rpc(format!("Failed to get block number: {:?}", e)))
    }

    // Fetch chain_id from RPC endpoint
    fn chain_id(&mut self) -> Result<u64> {
        // Create tokio runtime for async operations (Diesel is synchronous)
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| IndexerError::Runtime(e))?;

        // Create Alloy HTTP provider connected to the RPC URL
        let provider = alloy::providers::ProviderBuilder::new().connect_http(self.url.clone());
        // Use eth_chainId RPC method
        let chain_id = rt
            .block_on(provider.get_chain_id())
            .map_err(|e| IndexerError::Rpc(format!("Failed to get chain ID: {:?}", e)))?;
        Ok(chain_id.into())
    }

    // Fetch ERC20 Transfer event logs within a block range
    fn logs(&self, start_block: u64, end_block: u64) -> Result<impl IntoIterator<Item = Log>> {
        // Create tokio runtime for async operations (Diesel is synchronous)
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| IndexerError::Runtime(e))?;

        // Parse Transfer event signature as topic0 for log filtering
        let transfer_topic: alloy::primitives::FixedBytes<32> =
            TRANSFER_EVENT_SIGNATURE.parse().map_err(|e| {
                IndexerError::Parse(format!("Failed to parse transfer signature: {:?}", e))
            })?;

        // Build a log filter to query Transfer events
        let filter = Filter::new()
            .from_block(start_block) // Start block number (inclusive)
            .to_block(end_block) // End block number (inclusive)
            .address(self.token_address) // Filter by token contract address
            .event_signature(transfer_topic); // Filter by Transfer event signature (topic0)

        // Create Alloy HTTP provider connected to the RPC URL
        let provider = alloy::providers::ProviderBuilder::new().connect_http(self.url.clone());
        // Block on the async get_logs() call with the filter and return the logs
        // This converts the async operation to a synchronous one
        rt.block_on(provider.get_logs(&filter))
            .map_err(|e| IndexerError::Rpc(format!("Failed to get logs: {:?}", e)))
    }
}

// Initialize or update the sync table with a starting block number
// Returns true if the block number was updated, false if it was already higher
pub fn start_from(conn: &mut diesel::SqliteConnection, chain_id: u64, start: u64) -> Result<bool> {
    // Update the sync table if the current block number is less than (start - 1)
    // Using (start - 1) to start indexing from the 'start' block
    let start_block_value = start as i64 - 1;

    // Use upsert: insert if not exists, update if exists and block_number is less
    diesel::insert_into(schema::sync::table)
        .values((
            schema::sync::chain_id.eq(chain_id as i32),
            schema::sync::block_number.eq(start_block_value),
        ))
        .on_conflict(schema::sync::chain_id)
        .do_update()
        .set(schema::sync::block_number.eq(start_block_value))
        .execute(conn)?;

    Ok(true)
}

// Main event loop for continuous indexing
// This function will run indefinitely, fetching and processing blocks until interrupted
pub fn event_loop(
    _conn: &mut diesel::SqliteConnection, // DB connection
    _chain_id: u64,                       // Chain ID for DB operations
    _provider: impl LogsProvider,         // RPC provider
    _range_size: u64,                     // Num of blocks per iteration
) -> Result<()> {
    // TODO: Fetch last updated block from the db
    // TODO: Loop until interrupted
    // TODO: Fetch latest block from RPC
    // TODO: Process block ranges
    // TODO: Handle Transfer events
    // TODO: Update sync table

    Ok(())
}
