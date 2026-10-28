pub struct RegisterInput {
    pub name: String,
    pub password: String,
    pub email: String,
}

pub struct RegisterOutput {
    pub id: sqlx::types::Uuid,
    pub created_at: time::OffsetDateTime,
}

pub struct LoginInput {
    pub email: String,
    pub password: String,
}

pub struct LoginOutput {
    pub access_token: String,
    pub expires_at: usize,
}
