CREATE TABLE impact_snapshots (
    id SERIAL PRIMARY KEY,
    incident_id UUID NOT NULL REFERENCES incidents(id),
    ledger_sequence BIGINT NOT NULL,
    total_frozen_accounts BIGINT NOT NULL,
    total_frozen_trustlines BIGINT NOT NULL,
    total_bypassed_txs BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
