DROP INDEX IF EXISTS idx_url_mapping_user_id;

CREATE INDEX IF NOT EXISTS idx_short_code ON url_mapping (short_code) WHERE expiration_date IS NULL AND status = 'active';
CREATE INDEX IF NOT EXISTS idx_original_url ON url_mapping (original_url) WHERE expiration_date IS NULL AND status = 'active';
