-- Latest network facts observed by the indexer and the last time stored
-- state matched the freeze configuration read from the network.
CREATE TABLE network_observations (
    network_id UUID PRIMARY KEY REFERENCES networks(id),
    latest_network_ledger BIGINT NOT NULL,
    protocol_version INTEGER,
    observed_at TIMESTAMPTZ NOT NULL,
    last_reconciled_ledger BIGINT,
    last_reconciled_at TIMESTAMPTZ
);

-- Key history and active-episode lookups.
CREATE INDEX freeze_changes_key_idx ON freeze_changes (network_id, key_hash, ledger_sequence);
CREATE INDEX incidents_active_idx ON incidents (network_id) WHERE status = 'active';
