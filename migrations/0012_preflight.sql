CREATE TABLE preflight_observations (
    id SERIAL PRIMARY KEY,
    incident_id UUID NOT NULL REFERENCES incidents(id),
    ledger_sequence BIGINT NOT NULL,
    transaction_hash TEXT NOT NULL,
    result TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
