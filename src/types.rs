use alloy_primitives::{Address, B256, U256};

#[derive(Debug, Clone)]
pub struct TransferEvent {
    pub chain_id: u64,
    pub block_number: u64,
    pub tx_hash: B256,
    pub token_address: Address,
    pub from_addr: Address,
    pub to_addr: Address,
    pub value: U256,
    pub log_index: u64,
}
