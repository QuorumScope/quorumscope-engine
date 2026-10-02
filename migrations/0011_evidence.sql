CREATE TABLE evidence (
    id UUID PRIMARY KEY,
    incident_id UUID NOT NULL REFERENCES incidents(id),
    kind TEXT NOT NULL,
    data JSONB NOT NULL,
    canonical_xdr BYTEA,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
