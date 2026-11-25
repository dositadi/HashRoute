//! The JSON shapes every endpoint returns, so a frontend can handle all responses the same way.
//!
//! Success: `{ "data": <payload> }`
//!
//! Error: `{ "error": { "code": "not_found", "message": "link not found", "details": { ... } } }`
//!
//! `code` is a stable, machine-readable string to branch on; `message` is for people.
//! `details` only appears on validation errors and maps each field to its messages.

use std::collections::BTreeMap;

use axum::{
    Json,
    extract::{FromRequest, Request, rejection::JsonRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::json;
use validator::{Validate, ValidationErrors};

/// A successful response: `{ "data": ... }` with the given status.
pub fn ok<T: Serialize>(status: StatusCode, data: T) -> Response {
    (status, Json(json!({ "data": data }))).into_response()
}

#[derive(Debug, Clone)]
pub struct ApiError {
    pub status: StatusCode,
    pub code: &'static str,
    pub message: String,
    pub details: Option<BTreeMap<String, Vec<String>>>,
}

impl ApiError {
    pub fn new(status: StatusCode, code: &'static str, message: impl Into<String>) -> Self {
        Self { status, code, message: message.into(), details: None }
    }

    pub fn not_found() -> Self {
        Self::new(StatusCode::NOT_FOUND, "not_found", "resource not found")
    }

    pub fn method_not_allowed() -> Self {
        Self::new(StatusCode::METHOD_NOT_ALLOWED, "method_not_allowed", "method not allowed")
    }

    /// One entry per invalid field, each with every message that applies to it.
    pub fn validation(errors: &ValidationErrors) -> Self {
        let details = errors
            .field_errors()
            .into_iter()
            .map(|(field, errs)| {
                let messages = errs
                    .iter()
                    .map(|e| e.message.as_ref().map_or_else(|| e.code.to_string(), ToString::to_string))
                    .collect();
                (field.to_string(), messages)
            })
            .collect();

        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            code: "validation_failed",
            message: "some fields are invalid".to_owned(),
            details: Some(details),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let mut error = json!({ "code": self.code, "message": self.message });
        if let Some(details) = self.details {
            error["details"] = json!(details);
        }

        (self.status, Json(json!({ "error": error }))).into_response()
    }
}

/// A JSON body that is deserialized and then validated. Both kinds of failure come back in
/// the standard error shape instead of axum's plain-text rejections.
pub struct ValidJson<T>(pub T);

impl<S, T> FromRequest<S> for ValidJson<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate,
{
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state).await.map_err(|rejection| {
            let code = match rejection {
                JsonRejection::MissingJsonContentType(_) => "unsupported_media_type",
                _ => "invalid_body",
            };
            ApiError::new(rejection.status(), code, rejection.body_text())
        })?;

        value.validate().map_err(|e| ApiError::validation(&e))?;

        Ok(Self(value))
    }
}

pub async fn not_found_fallback() -> ApiError {
    ApiError::not_found()
}

pub async fn method_not_allowed_fallback() -> ApiError {
    ApiError::method_not_allowed()
}
