-- Create schema for indexer syncing
CREATE TABLE ingestion_checkpoints (
    network_id UUID NOT NULL REFERENCES networks(id),
    stream TEXT NOT NULL,
    last_complete_ledger BIGINT NOT NULL,
    last_ledger_hash BYTEA,
    updated_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (network_id, stream)
);

CREATE TABLE config_snapshots (
    id UUID PRIMARY KEY,
    network_id UUID NOT NULL REFERENCES networks(id),
    ledger_sequence BIGINT NOT NULL,
    ledger_hash BYTEA,
    config_setting_id TEXT NOT NULL,
    raw_entry_xdr BYTEA NOT NULL,
    parsed_json JSONB NOT NULL,
    source_kind TEXT NOT NULL,
    observed_timestamp TIMESTAMPTZ NOT NULL
);

CREATE TABLE ledger_keys (
    network_id UUID NOT NULL REFERENCES networks(id),
    key_hash BYTEA NOT NULL,
    canonical_key_xdr BYTEA NOT NULL,
    key_kind TEXT NOT NULL,
    decoded_json JSONB NOT NULL,
    first_observed_ledger BIGINT NOT NULL,
    last_observed_ledger BIGINT NOT NULL,
    PRIMARY KEY (network_id, key_hash)
);

CREATE TABLE freeze_changes (
    id UUID PRIMARY KEY,
    network_id UUID NOT NULL REFERENCES networks(id),
    ledger_sequence BIGINT NOT NULL,
    key_hash BYTEA NOT NULL,
    action TEXT NOT NULL,
    result TEXT NOT NULL,
    evidence_ref TEXT,
    created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE current_frozen_keys (
    network_id UUID NOT NULL REFERENCES networks(id),
    key_hash BYTEA NOT NULL,
    active_since BIGINT NOT NULL,
    last_changed BIGINT NOT NULL,
    evidence_ref TEXT,
    PRIMARY KEY (network_id, key_hash)
);

CREATE TABLE bypass_changes (
    id UUID PRIMARY KEY,
    network_id UUID NOT NULL REFERENCES networks(id),
    ledger_sequence BIGINT NOT NULL,
    tx_hash TEXT NOT NULL,
    action TEXT NOT NULL,
    result TEXT NOT NULL,
    evidence_ref TEXT,
    created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE current_bypasses (
    network_id UUID NOT NULL REFERENCES networks(id),
    tx_hash TEXT NOT NULL,
    active_since BIGINT NOT NULL,
    last_changed BIGINT NOT NULL,
    evidence_ref TEXT,
    PRIMARY KEY (network_id, tx_hash)
);
