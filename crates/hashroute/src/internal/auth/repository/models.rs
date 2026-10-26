use time::OffsetDateTime;

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct User {
    pub id: sqlx::types::Uuid,
    pub name: String,
    pub email: String,
    pub password_hash: String,
    pub activated: bool,
    pub activated_at: OffsetDateTime,
    pub created_at: OffsetDateTime,
}

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct UserCreateResponse {
    pub id: sqlx::types::Uuid,
    pub created_at: OffsetDateTime,
}

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct UserCreateInput {
    pub name: String,
    pub email: String,
    pub password_hash: String,
}
