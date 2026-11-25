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
    pub expiration_date: Option<time::OffsetDateTime>,
}

#[derive(Debug, Clone, validator::Validate, Deserialize)]
pub struct NewURL {
    pub user_id: sqlx::types::Uuid,
    pub short_code: String,
    #[validate(url)]
    pub original_url: String,
    #[validate(custom(function=validate_status))]
    pub status: String,
}

pub type Status = &'static str;

pub const STATUS_ACTIVE: Status = "active";
pub const STATUS_EXPIRED: Status = "expired";

pub fn validate_status(status: &str) -> Result<(), ValidationError> {
    if status.eq(STATUS_ACTIVE) || status.eq(STATUS_EXPIRED) {
        return Ok(());
    }
    Err(ValidationError::new("invalid status"))
}

/// Body of `POST /tiny/v1/links`.
#[derive(Debug, Deserialize, validator::Validate)]
pub struct CreateLinkPayload {
    #[validate(length(min = 1, max = 2048, message = "url must be 1 to 2048 characters"))]
    pub url: String,
}

/// What the API returns for a link. Dates are RFC 3339 strings and `short_url` is ready to
/// display or copy.
#[derive(Debug, Clone, Serialize)]
pub struct LinkView {
    pub id: sqlx::types::Uuid,
    pub short_code: String,
    pub short_url: String,
    pub original_url: String,
    pub status: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: time::OffsetDateTime,
    #[serde(with = "time::serde::rfc3339::option")]
    pub expires_at: Option<time::OffsetDateTime>,
}

impl LinkView {
    pub fn new(url: URL, public_base_url: &str) -> Self {
        Self {
            short_url: format!("{public_base_url}/{}", url.short_code),
            id: url.id,
            short_code: url.short_code,
            original_url: url.original_url,
            status: url.status,
            created_at: url.created_at,
            expires_at: url.expiration_date,
        }
    }
}
