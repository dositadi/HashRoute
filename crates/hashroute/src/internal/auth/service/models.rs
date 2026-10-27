use axum::{ Json, http::StatusCode, response::IntoResponse };
use serde::{ Deserialize, Serialize };
use serde_json::json;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Claim {
    pub id: String,
    pub sub: String,
    pub exp: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthBody {
    pub access_token: String,
    pub token_type: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthPayoad {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug)]
pub enum AuthError {
    WrongCredentails,
    MissingCredentials,
    TokenCreation,
    InvalidToken,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "invalid token"),
            AuthError::MissingCredentials => (StatusCode::UNAUTHORIZED, "missing credentials"),
            AuthError::TokenCreation =>
                (StatusCode::INTERNAL_SERVER_ERROR, "something wrong happened"),
            AuthError::WrongCredentails => (StatusCode::UNAUTHORIZED, "wrong credentials"),
        };

        let body = Json(json!({
            "error": message
        }));

        (status, body).into_response()
    }
}
