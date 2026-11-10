use axum::{Json, Router, extract::{Path, State}, http::StatusCode, response::IntoResponse, routing::{get, post}};
use serde::Serialize;
use serde_json::json;

use crate::internal::{auth::service::models::AccessClaims, config::app_state::AppState, routehasher::{domain::errors::URLDBError, repository::repo::Repository, service::handler_service::Service}};


type TinyUrlResponse<T> = Result<T, URLDBError>;

fn success_response<T>(resp: T, status_code: StatusCode) -> (StatusCode, impl IntoResponse) where T: Serialize + Send {
    let code = status_code.as_u16();
    let body = Json(json!(
        {
            "code": code,
            "body": resp
        }
    ));

    (status_code, body)
}
pub struct TinyUrlHandler;

impl TinyUrlHandler {
    pub fn router() -> axum::Router<AppState> {
        Router::new()
        .route("/generate/{url}", post(generate_tiny_url))
        .route("/list-all/{id}", get)
        .route("/get/{id}", get(handler))
        .route("/deactivate", post(handler))
    }
}

async fn generate_tiny_url(State(state): State<AppState>,Path(url): Path<String>, claims: AccessClaims) -> TinyUrlResponse<(StatusCode,impl IntoResponse)>  {
    let result = Service::new(Repository::new(state.repository.postgres.db_pool)).generate(&url, claims).await?;

    Ok(success_response(result,StatusCode::OK))
}
