# rust-indexer

A minimal **Rust-based on-chain indexer** that extracts ERC20 `Transfer` events from an Ethereum RPC endpoint and stores them in a local SQLite database.

---

## Project Structure

```
src/
├── main.rs       # Entry point & orchestration (tokio tasks)
├── config.rs     # Environment configuration (.env)
├── indexer.rs    # Core indexing logic (fetch + parse)
└── storage.rs    # Database operations using Diesel
```

---

## Scope

### Core Features

- Connect to an Ethereum-compatible RPC (`HTTP` or `WS`)
- Fetch and decode ERC20 `Transfer` logs
- Persist decoded events into SQLite
- Automatically resume from the last indexed block (state checkpoint)
- Basic error handling & retry logic

### Out of Scope (for now)

- REST API or query layer
- Real-time subscriptions (WebSocket)
- Multi-chain or multi-token indexing
- Reorg or confirmation-depth handling

---

## Setup

1. Copy `.env.example` and configure environment variables:

   ```bash
   cp .env.example .env
   ```

   Example:

   ```env
   RPC_URL=https://ethereum-sepolia-rpc.publicnode.com
   START_BLOCK=0
   DB_PATH=indexer.db
   CHAIN_ID=11155111
   ```

2. Build and run:
   ```bash
   cargo build --release
   cargo run
   ```

---

## Database Schema

```sql
CREATE TABLE sync (
    chain_id INTEGER NOT NULL,
    block_number INTEGER NOT NULL,
    PRIMARY KEY (chain_id)
);

CREATE TABLE transfers (
    chain_id INTEGER NOT NULL,
    block_number INTEGER NOT NULL,
    tx_hash CHAR(66) NOT NULL,
    token_address CHAR(42) NOT NULL,
    from_addr CHAR(42) NOT NULL,
    to_addr CHAR(42) NOT NULL,
    value NUMERIC NOT NULL,
    log_index INTEGER NOT NULL,
    PRIMARY KEY (chain_id, tx_hash, log_index)
);

CREATE INDEX idx_block  ON transfers(chain_id, block_number);
CREATE INDEX idx_token  ON transfers(chain_id, token_address);
CREATE INDEX idx_from   ON transfers(from_addr);
CREATE INDEX idx_to     ON transfers(to_addr);
```

---

## Dependencies

- [`tokio`](https://crates.io/crates/tokio) — async runtime
- [`alloy`](https://crates.io/crates/alloy) — Ethereum RPC client
- [`diesel`](https://crates.io/crates/diesel) — ORM / SQLite layer
- [`dotenvy`](https://crates.io/crates/dotenvy) — environment configuration
- [`hex`](https://crates.io/crates/hex) — hex encoding / decoding

---

## Indexing Flow

```text
 ┌────────────┐        ┌──────────────┐        ┌────────────┐
 │  Fetcher   │────→──▶│    Parser    │────→──▶│  Storage   │
 │ (RPC logs) │  mpsc  │ (ABI decode) │  mpsc  │  (SQLite)  │
 └────────────┘        └──────────────┘        └────────────┘
```

Each stage runs as a separate Tokio task connected via `mpsc` channels,  
allowing concurrent fetching, decoding, and writing.

---

## Current Status

- [ ] `.env` configuration & project skeleton
- [ ] SQLite schema migration
- [ ] Fetch `Transfer` logs via `alloy`
- [ ] Decode topics into structured events
- [ ] Insert into DB with Diesel
- [ ] Log progress and exit

---

## Roadmap

- Add `/health` and `/transfers?addr=` REST API with `axum`
- Reorg handling (`CONFIRMATIONS`)
- Batched inserts for performance
- Multi-token or multi-chain support
- Prometheus metrics (QPS, block height)
- Optional Postgres backend

---

## Notes

- Async task orchestration with `tokio::spawn` and `mpsc` channels
- Practical use of `alloy` RPC APIs for Ethereum interaction
- Clean modular design leveraging Rust's ownership model
- Extensible architecture for production-grade indexers

---
