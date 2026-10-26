pub const CREATEUSER: &str = r#"
INSERT INTO users (
    name, email, password_hash, 
) VALUES (
    $1, $2, $3,
)
RETURNING id, created_at;
"#;

pub const GETUSERBYID: &str = r#"
SELECT * FROM users
WHERE id = $1;
"#;

pub const GETUSERBYEMAIL: &str = r#"
SELECT * FROM users
WHERE id = $1;
"#;