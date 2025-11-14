use crate::schema;
use alloy::primitives::Address; // Alloy primitives for Ethereum types (Address, B256, U256, etc.)
use alloy::providers::Provider; // Provider trait for RPC operations (get_block_number, get_logs, etc.)
use alloy::rpc::types::Filter; // Filter type for constructing log queries
use alloy::rpc::types::eth::Log; // Log type representing Ethereum event logs
use alloy::transports::http::reqwest::Url; // URL type for HTTP RPC endpoints
use anyhow::Result; // Error handling utilities
use diesel::prelude::*; // Diesel ORM prelude (Connection, QueryDsl, etc.) // Database schema definitions (sync, transfers tables)

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
        // Create a single-threaded tokio runtime for async operations
        // This is needed because Diesel operations are synchronous
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all() // Enable all tokio features (timer, io, etc.)
            .build()?; // Build the runtime, propagate errors

        // Create an Alloy HTTP provider connected to the RPC URL (Alloy provider)
        let provider = alloy::providers::ProviderBuilder::new().connect_http(self.url.clone());

        // Block on the async get_block_number() call and return the result
        // This converts the async operation to a synchronous one
        Ok(rt.block_on(provider.get_block_number())?)
    }

    fn chain_id(&mut self) -> Result<u64> {
        // Create tokio runtime for async operations (Diesel is synchronous)
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let provider = alloy::providers::ProviderBuilder::new().connect_http(self.url.clone());
        // Use eth_chainId RPC method
        let chain_id = rt.block_on(provider.get_chain_id())?;
        Ok(chain_id.into())
    }

    // Fetch ERC20 Transfer event logs within a block range
    fn logs(&self, start_block: u64, end_block: u64) -> Result<impl IntoIterator<Item = Log>> {
        // Create a single-threaded tokio runtime for async operations
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all() // Enable all tokio features
            .build()?; // Build the runtime

        // Parse the Transfer event signature into a 32-byte fixed array
        // This will be used as topic0 in the log filter
        let transfer_topic: alloy::primitives::FixedBytes<32> = TRANSFER_EVENT_SIGNATURE.parse()?;

        // Build a log filter to query Transfer events
        let filter = Filter::new()
            .from_block(start_block) // Start block number (inclusive)
            .to_block(end_block) // End block number (inclusive)
            .address(self.token_address) // Filter by token contract address
            .event_signature(transfer_topic); // Filter by Transfer event signature (topic0)

        // Create an Alloy HTTP provider connected to the RPC URL
        let provider = alloy::providers::ProviderBuilder::new().connect_http(self.url.clone());

        // Block on the async get_logs() call with the filter and return the logs
        // This converts the async operation to a synchronous one
        Ok(rt.block_on(provider.get_logs(&filter))?)
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
