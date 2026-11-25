use axum::{http::StatusCode, response::IntoResponse};
use tracing::error;

use crate::internal::platform::api::ApiError;

#[derive(Debug, thiserror::Error)]
pub enum URLDBError {
    #[error("short_code exists already")]
    ShortCodeExists,
    #[error("network error")]
    IO(String),
    #[error("timeout")]
    Timeout,
    #[error("something wrong happened")]
    Internal,
    #[error("status is not valid")]
    InvalidStatus,
    #[error("link not found")]
    NotFound,
    #[error("link not found")]
    NoRowAffected,
    #[error("invalid credential: {0}")]
    InvalidCredential(String),
    #[error("{0}")]
    InvalidUrl(String),
    #[error("invalid link id")]
    InvalidId,
}

impl From<URLDBError> for ApiError {
    fn from(err: URLDBError) -> Self {
        let (status, code) = match &err {
            URLDBError::IO(why) => {
                error!(error=?why, "IO error in URL DB");
                (StatusCode::SERVICE_UNAVAILABLE, "service_unavailable")
            }
            URLDBError::ShortCodeExists | URLDBError::Internal => {
                (StatusCode::INTERNAL_SERVER_ERROR, "internal_error")
            }
            URLDBError::Timeout => (StatusCode::SERVICE_UNAVAILABLE, "timeout"),
            URLDBError::InvalidStatus => (StatusCode::UNPROCESSABLE_ENTITY, "invalid_status"),
            URLDBError::NotFound | URLDBError::NoRowAffected => (StatusCode::NOT_FOUND, "not_found"),
            URLDBError::InvalidCredential(_) => (StatusCode::UNAUTHORIZED, "invalid_token"),
            URLDBError::InvalidUrl(_) => (StatusCode::UNPROCESSABLE_ENTITY, "invalid_url"),
            URLDBError::InvalidId => (StatusCode::BAD_REQUEST, "invalid_id"),
        };

        ApiError::new(status, code, err.to_string())
    }
}

impl IntoResponse for URLDBError {
    fn into_response(self) -> axum::response::Response {
        ApiError::from(self).into_response()
    }
}
