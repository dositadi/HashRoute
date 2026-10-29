use serde::{ Serialize };
use uuid::Uuid;

pub struct RegisterInput {
    pub name: String,
    pub password: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RegisterOutput {
    pub id: Uuid,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: time::OffsetDateTime,
}

pub struct LoginInput {
    pub email: String,
    pub password: String,
}

pub struct Token {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: usize,
}
