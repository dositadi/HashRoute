#[derive(Debug, Clone)]
pub struct MappedUrl {
    pub user_id: sqlx::types::Uuid,
    pub short_code: String,
    pub original_url: String,
    pub status: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MappedUrlResp {
    pub id: String,
    pub short_code: String,
    pub original_url: String,
    pub created_at: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct URL {
    pub id: sqlx::types::Uuid,
    pub short_code: String,
    pub original_code: String,
    pub status: String,
}
