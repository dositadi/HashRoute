-- short_code already has a unique index from its UNIQUE constraint, and the old partial
-- indexes only covered rows with no expiration date. Replace them with an index for
-- listing a user's URLs.
DROP INDEX IF EXISTS idx_short_code;
DROP INDEX IF EXISTS idx_original_url;

CREATE INDEX IF NOT EXISTS idx_url_mapping_user_id ON url_mapping (user_id, created_at DESC);
