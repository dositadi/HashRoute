-- Add down migration script here
BEGIN;

DROP INDEX IF EXISTS idx_original_url;
DROP INDEX IF EXISTS idx_short_code;
DROP INDEX IF EXISTS url_mapping;

COMMIT;