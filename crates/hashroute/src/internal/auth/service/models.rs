use axum::{ Json, http::StatusCode, response::IntoResponse };
use jsonwebtoken::{ DecodingKey, EncodingKey };
use serde::{ Deserialize, Serialize };
use serde_json::json;

use crate::internal::auth::repository::error::DBError;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Claims {
    pub id: String,
    pub token_type: String,
    pub sub: String,
    pub exp: usize,
    pub iss: String,
    pub iat: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JwtAccessTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_at: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthPayload {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug)]
pub enum AuthError {
    InvalidPassword,
    WrongCredentails,
    MissingCredentials,
    TokenCreation,
    InvalidToken,
    InvalidID,
    TokenExpired,
    Internal,
    TimeOut,
    Network(String),
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
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "invalid token"),
            AuthError::MissingCredentials => (StatusCode::UNAUTHORIZED, "missing credentials"),
            AuthError::TokenCreation | AuthError::Internal | AuthError::Network(_) =>
                (StatusCode::INTERNAL_SERVER_ERROR, "something wrong happened"),
            AuthError::WrongCredentails => (StatusCode::UNAUTHORIZED, "wrong credentials"),
            AuthError::InvalidID => (StatusCode::UNAUTHORIZED, "invalid id"),
            AuthError::TokenExpired => (StatusCode::UNAUTHORIZED, "expired token"),
            AuthError::InvalidPassword => (StatusCode::UNAUTHORIZED, "invalid password"),
            AuthError::TimeOut => (StatusCode::REQUEST_TIMEOUT, "network error"),
            AuthError::InvalidUser => (StatusCode::UNAUTHORIZED, "this user does not exist"),
        };

        let body = Json(json!({
            "error": message
        }));

        (status, body).into_response()
    }
}

#[derive(Debug)]
pub struct Keys {
    pub encoding_key: EncodingKey,
    pub decoding_key: DecodingKey,
}

impl Keys {
    pub fn new(secret: &[u8]) -> Keys {
        Keys {
            encoding_key: EncodingKey::from_secret(secret),
            decoding_key: DecodingKey::from_secret(secret),
        }
    }
}
