CREATE TABLE bypass_state (
    id SERIAL PRIMARY KEY,
    incident_id UUID NOT NULL REFERENCES incidents(id),
    ledger_sequence BIGINT NOT NULL,
    tx_hash TEXT NOT NULL,
    decoded_json JSONB NOT NULL,
    canonical_xdr BYTEA NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
