use axum::{ Json, http::StatusCode, response::IntoResponse };
use serde_json::json;

pub enum URLHashError {
    InvalidURL,
    Internal,
}

impl IntoResponse for URLHashError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            Self::Internal => (StatusCode::INTERNAL_SERVER_ERROR, "something wrong happened"),
            Self::InvalidURL => (StatusCode::UNPROCESSABLE_ENTITY, "invalid url"),
        };

        let code = &status.as_u16();

        let body = Json(json!({
            "error":message,
            "code": *code
        }));

        (status, body).into_response()
    }
}
