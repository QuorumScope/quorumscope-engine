CREATE TABLE networks (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    passphrase TEXT NOT NULL
);
