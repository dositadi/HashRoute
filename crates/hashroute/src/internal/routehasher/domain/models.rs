use serde::{Deserialize, Serialize};
use validator::ValidationError;

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct URL {
    pub id: sqlx::types::Uuid,
    pub user_id: sqlx::types::Uuid,
    pub short_code: String,
    pub original_url: String,
    pub status: String,
    pub created_at: time::OffsetDateTime,
    pub expiration_time: Option<time::OffsetDateTime>
}

#[derive(Debug, Clone, validator::Validate, Deserialize)]
pub struct  NewURL {
    pub user_id: sqlx::types::Uuid,
    pub short_code: String,
    #[validate(url)]
    pub original_url: String,
    #[validate(custom(function=validate_status))]
    pub status: String
}

pub type Status = &'static str;

pub const STATUS_ACTIVE: Status = "active";
pub const STATUS_EXPIRED: Status = "expired";

pub fn validate_status(status: &str) -> Result<(), ValidationError> {
    if status.eq(STATUS_ACTIVE) || status.eq(STATUS_EXPIRED) {
        return Ok(())
    }
    Err(ValidationError::new("invalid status"))
}
