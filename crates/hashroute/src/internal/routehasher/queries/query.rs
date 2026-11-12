pub const INSERT_URL: &str =
    r#"
INSERT INTO url_mapping (
    user_id, short_code, original_url, status
)  VALUES  (
    $1, $2, $3, $4
)
RETURNING *;
"#;

pub const LIST_URLS: &str =
    r#"
SELECT *
FROM url_mapping
WHERE user_id = $1
AND expiration_date IS NULL
ORDER BY created_at DESC;
"#;

pub const GET_URL_BY_CODE: &str =
    r#"
SELECT *
FROM url_mapping
WHERE short_code = $1
AND expiration_date IS NULL
ORDER BY created_at DESC;
"#;

pub const GET_URL_BY_ID: &str =
    r#"
SELECT *
FROM url_mapping
WHERE id = $1
AND expiration_date IS NULL
ORDER BY created_at DESC;
"#;

pub const DEACTIVATE_URL: &str =
    r#"
UPDATE url_mapping
SET status = 'inactive', expiration_date = NOW()
WHERE id = $1 AND expiration_date IS NULL;
"#;
