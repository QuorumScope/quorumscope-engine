CREATE TABLE incidents (
    id UUID PRIMARY KEY,
    network_id UUID NOT NULL REFERENCES networks(id),
    basis TEXT NOT NULL,
    opened_ledger BIGINT NOT NULL,
    opened_close_time TIMESTAMPTZ,
    closed_ledger BIGINT,
    closed_close_time TIMESTAMPTZ,
    status TEXT NOT NULL
);

CREATE TABLE incident_events (
    id SERIAL PRIMARY KEY,
    incident_id UUID NOT NULL REFERENCES incidents(id),
    ledger_sequence BIGINT NOT NULL,
    kind TEXT NOT NULL,
    evidence_ref TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE metrics (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    value DOUBLE PRECISION NOT NULL,
    labels JSONB NOT NULL DEFAULT '{}',
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
