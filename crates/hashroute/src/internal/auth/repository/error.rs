use axum::{ Json, http::StatusCode, response::IntoResponse };
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DBError {
    #[error("not found")]
    NotFound,
    #[error("email already exists")]
    EmailConflict,
    #[error("something wrong happened")]
    Internal,
    #[error("timeout")]
    TimeOut,
    #[error("connection failure")] Io(String),
}

impl IntoResponse for DBError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            DBError::EmailConflict => { (StatusCode::CONFLICT, self.to_string()) }
            DBError::Internal => { (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()) }
            DBError::Io(_) => { (StatusCode::SERVICE_UNAVAILABLE, self.to_string()) }
            DBError::NotFound => { (StatusCode::NOT_FOUND, self.to_string()) }
            DBError::TimeOut => { (StatusCode::REQUEST_TIMEOUT, self.to_string()) }
        };

        let body = Json(json!({
            "error": message
        }));

        (status, body).into_response()
    }
}
