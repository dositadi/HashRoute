use axum::{ Json, http::StatusCode, response::IntoResponse };
use serde_json::json;

pub enum DBError {
    NotFound,
    EmailConflict,
    Internal,
    TimeOut,
    Io(String),
}

impl IntoResponse for DBError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            DBError::EmailConflict => { (StatusCode::CONFLICT, "email already exists") }
            DBError::Internal => { (StatusCode::INTERNAL_SERVER_ERROR, "something wrong happened") }
            DBError::Io(_) => { (StatusCode::SERVICE_UNAVAILABLE, "connection failure") }
            DBError::NotFound => { (StatusCode::NOT_FOUND, "not found") }
            DBError::TimeOut => { (StatusCode::REQUEST_TIMEOUT, "timeout occurred") }
        };

        let body = Json(json!({
            "error": message
        }));

        (status, body).into_response()
    }
}
