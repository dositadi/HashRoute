use axum::{ Json, http::StatusCode, response::IntoResponse };
use serde_json::json;
use thiserror::Error;

use crate::internal::auth::repository::error::DBError;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("invalid password")]
    InvalidPassword,
    #[error("invalid credentials")]
    WrongCredentails,
    #[error("missing credentials")]
    MissingCredentials,
    #[error("something wrong happened")]
    TokenCreation,
    #[error("invalid token")]
    InvalidToken,
    #[error("invalid id")]
    InvalidID,
    #[error("token expired")]
    TokenExpired,
    #[error("something wrong happened")]
    Internal,
    #[error("timeout")]
    TimeOut,
    #[error("network error")] Network(String),
    #[error("invalid user")]
    InvalidUser,
}

impl From<DBError> for AuthError {
    fn from(value: DBError) -> Self {
        match value {
            DBError::NotFound => AuthError::InvalidUser,
            DBError::Io(why) => AuthError::Network(why),
            DBError::TimeOut => AuthError::TimeOut,
            _ => AuthError::Internal,
        }
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            AuthError::TokenCreation | AuthError::Internal | AuthError::Network(_) =>
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AuthError::TimeOut => (StatusCode::REQUEST_TIMEOUT, self.to_string()),
            AuthError::InvalidToken | AuthError::MissingCredentials | AuthError::WrongCredentails | AuthError::InvalidID | AuthError::TokenExpired | AuthError::InvalidPassword | AuthError::InvalidUser => (StatusCode::UNAUTHORIZED, self.to_string()),
        };

        let body = Json(json!({
            "error": message
        }));

        (status, body).into_response()
    }
}
