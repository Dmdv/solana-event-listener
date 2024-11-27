CREATE DATABASE point_system;

\c point_system;

CREATE TABLE minted_gorple
(
    id             SERIAL PRIMARY KEY,
    dst_address    TEXT      NOT NULL,
    amount         DECIMAL   NOT NULL,
    fee            DECIMAL   NOT NULL,
    src_tx_hash    TEXT      NOT NULL,
    src_chain_id   BIGINT    NOT NULL,
    nonce          BIGINT    NOT NULL,
    dst_tx_hash    TEXT      NOT NULL,
    dst_block_slot BIGINT    NOT NULL,
    dst_block_time TIMESTAMP NOT NULL
);

CREATE UNIQUE INDEX idx_src_chain_id_src_tx_hash ON minted_gorple (src_chain_id, src_tx_hash);
