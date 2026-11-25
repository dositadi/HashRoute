use axum::{ http::StatusCode, response::IntoResponse };
use thiserror::Error;

use crate::internal::{ auth::repository::error::DBError, platform::api::ApiError };

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
    #[error("invalid token credential")]
    UnAuthorized,
    #[error("bearer token header missing")]
    BearerToken,
    #[error("x-refresh header missing")]
    RefreshToken,
    #[error("an account with this email already exists")]
    EmailTaken,
}

impl From<DBError> for AuthError {
    fn from(value: DBError) -> Self {
        match value {
            DBError::NotFound => AuthError::InvalidUser,
            DBError::Io(why) => AuthError::Network(why),
            DBError::TimeOut => AuthError::TimeOut,
            DBError::EmailConflict => AuthError::EmailTaken,
            _ => AuthError::Internal,
        }
    }
}

impl From<AuthError> for ApiError {
    fn from(err: AuthError) -> Self {
        let unauthorized = StatusCode::UNAUTHORIZED;
        let (status, code, message) = match &err {
            AuthError::TokenCreation | AuthError::Internal => {
                (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", err.to_string())
            }
            AuthError::Network(_) => {
                (StatusCode::SERVICE_UNAVAILABLE, "service_unavailable", err.to_string())
            }
            AuthError::TimeOut => (StatusCode::SERVICE_UNAVAILABLE, "timeout", err.to_string()),
            // Same answer for an unknown email and a wrong password, so the API does not
            // reveal which emails are registered.
            AuthError::InvalidPassword | AuthError::InvalidUser => {
                (unauthorized, "invalid_credentials", "invalid email or password".to_owned())
            }
            AuthError::TokenExpired => (unauthorized, "token_expired", err.to_string()),
            AuthError::BearerToken | AuthError::RefreshToken => {
                (unauthorized, "missing_token", err.to_string())
            }
            AuthError::InvalidToken
            | AuthError::MissingCredentials
            | AuthError::WrongCredentails
            | AuthError::InvalidID
            | AuthError::UnAuthorized => (unauthorized, "invalid_token", err.to_string()),
            AuthError::EmailTaken => (StatusCode::CONFLICT, "email_taken", err.to_string()),
        };

        ApiError::new(status, code, message)
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        ApiError::from(self).into_response()
    }
}
