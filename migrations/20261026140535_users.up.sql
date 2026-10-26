BEGIN;

CREATE EXTENSION IF NOT EXISTS CITEXT;

-- Add up migration script here
CREATE TABLE IF NOT EXISTS users (
    id uuid NOT NULL PRIMARY KEY gen_random_uuid(),
    name TEXT NOT NULL,
    email CITEXT NOT NULL UNIQUE,
    password_hash BYTEA NOT NULL,  
    activated BOOLEAN DEFAULT FALSE,
    activated_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_name ON users (name) WHERE activated IS NOT NULL;
CREATE INDEX idx_email ON users (email) WHERE activated IS NOT NULL;

COMMIT;