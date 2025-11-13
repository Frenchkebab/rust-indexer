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

