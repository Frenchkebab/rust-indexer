// @generated automatically by Diesel CLI.

diesel::table! {
    sync (chain_id) {
        chain_id -> Integer,
        block_number -> BigInt,
    }
}

diesel::table! {
    transfers (chain_id, tx_hash, log_index) {
        chain_id -> Integer,
        block_number -> BigInt,
        tx_hash -> Text,
        token_address -> Text,
        from_addr -> Text,
        to_addr -> Text,
        value -> Text,
        log_index -> BigInt,
    }
}

diesel::allow_tables_to_appear_in_same_query!(sync, transfers,);
