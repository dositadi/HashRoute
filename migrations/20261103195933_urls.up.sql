-- Add up migration script here
BEGIN;

CREATE TABLE IF NOT EXISTS url_mapping (
    id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL FOREIGN KEY ON DELETE CASCADE,
    short_code TEXT UNIQUE NOT NULL,
    original_url TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'expired')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expiration_date TIMESTAMPTZ 
);

CREATE INDEX IF NOT EXISTS idx_short_code ON url_mapping (short_code) WHERE expiration_date IS NULL AND status = 'active';
CREATE INDEX IF NOT EXISTS idx_original_url ON url_mapping (original_url) WHERE expiration_date IS NULL AND status = 'active';

COMMIT;

