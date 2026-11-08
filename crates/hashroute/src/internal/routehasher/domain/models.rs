use serde::Deserialize;
use validator::ValidationError;

#[derive(Debug, Clone, sqlx::FromRow)]
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
    #[validate(required)]
    pub user_id: sqlx::types::Uuid,
    pub short_code: String,
    #[validate(required, url)]
    pub original_url: String,
    #[validate(required, custom(function=validate_status))]
    pub status: String
}

type Status = &'static str;

const STATUS_ACTIVE: Status = "active";
const STATUS_EXPIRED: Status = "expired";

fn validate_status(status: &str) -> Result<(), ValidationError> {
    if status.eq(STATUS_ACTIVE) || status.eq(STATUS_EXPIRED) {
        return Ok(())
    }
    Err(ValidationError::new("invalid status"))
}
