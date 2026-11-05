pub const INSERT_URL: &str =
    r#"
INSERT INTO url_mapping (
    user_id, short_code, original_url, status
)  VALUES  (
    $1, $2, $3, $4
)
RETURNING id, short_code, original_url, created_at;
"#;

pub const LIST_URLS: &str =
    r#"
SELECT id, short_code, original_url, status, created_at, expiration_date
FROM url_mapping
WHERE user_id = $1
AND expiration_date IS NULL
ORDER BY created_at DESC true;
"#;

pub const GET_URL_BY_CODE: &str =
    r#"
SELECT id, short_code, original_url, status, created_at, expiration_date
FROM url_mapping
WHERE short_code = $1
AND expiration_date IS NULL
ORDER BY created_at DESC true;
"#;

pub const GET_URL_BY_ID: &str =
    r#"
SELECT id, short_code, original_url, status, created_at, expiration_date
FROM url_mapping
WHERE id = $1
AND expiration_date IS NULL
ORDER BY created_at DESC true;
"#;

pub const DEACTIVATE_URL: &str =
    r#"
UPDATE url_mapping
SET status = 'active',
SET expiration_date = NOW()
WHERE id = $1
AND expiration_date IS NULL;
"#;
