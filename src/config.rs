pub struct Config {
    pub rpc_url: String,
    pub start_block: u64,
    pub db_path: String,
    pub chain_id: u64,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Config {
            rpc_url: std::env::var("RPC_URL")
                .unwrap_or_else(|_| "https://eth.llamarpc.com".to_string()),
            start_block: std::env::var("START_BLOCK")
                .unwrap_or_else(|_| "0".to_string())
                .parse()?,
            db_path: std::env::var("DB_PATH").unwrap_or_else(|_| "indexer.db".to_string()),
            chain_id: std::env::var("CHAIN_ID")
                .unwrap_or_else(|_| "11155111".to_string())
                .parse()?,
        })
    }
}
