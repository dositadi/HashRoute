use axum::{ Json, http::StatusCode, response::IntoResponse };
use serde_json::json;
use tracing::error;

#[derive(Debug, thiserror::Error)]
pub enum URLDBError {
    #[error("short_code exists already")]
    ShortCodeExists,
    #[error("network error")] IO(String),
    #[error("timeout")]
    Timeout,
    #[error("something wrong happened")]
    Internal,
    #[error("status is not  valid")]
    InvalidStatus,
    #[error("not found")]
    NotFound,
    #[error("no row affected")]
    NoRowAffected,
    #[error("invalid credential")]
    InvalidCredential(String),
}

impl IntoResponse for URLDBError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            Self::IO(why) => {
                error!(error=?why,"IO error in URL DB");
                (StatusCode::SERVICE_UNAVAILABLE, self.to_string())
            }
            Self::ShortCodeExists => (StatusCode::CONFLICT, self.to_string()),
            Self::Timeout => (StatusCode::REQUEST_TIMEOUT, self.to_string()),
            Self::Internal => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            Self::InvalidStatus => (StatusCode::UNPROCESSABLE_ENTITY, self.to_string()),
            Self::NotFound | Self::NoRowAffected => (StatusCode::NOT_FOUND, self.to_string()),
            Self::InvalidCredential(why) => {
                let mut message =  self.to_string();
                message.push_str(format!(": {}", why).as_str());

                (StatusCode::UNAUTHORIZED, message.to_owned())
            }
        };

        let code = &status.as_u16();

        let body = Json(json!({
            "error": message,
            "code": *code,
        }));

        (status, body).into_response()
    }
}
