pub struct RegisterInput {
    pub name: String,
    pub password: String,
    pub email: String,
}

pub struct RegisterOutput {
    pub id: sqlx::types::Uuid,
    pub created_at: time::OffsetDateTime,
}
