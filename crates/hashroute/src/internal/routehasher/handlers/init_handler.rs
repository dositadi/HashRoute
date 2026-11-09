use axum::{Router, extract::{Path, State}, handler::HandlerService, http::StatusCode, routing::get};
use tracing_subscriber::fmt::format::Json;

use crate::internal::{auth::service::models::AccessClaims, config::app_state::AppState, routehasher::{domain::{errors::URLDBError, models::{NewURL, URL}}, repository::repo::Repository, service::{create_hash::TinyUrl, handler_service::{HandlerService, Service}}}};


type TinyUrlResponse<T> = Result<T, URLDBError>;
pub struct TinyUrlHandler;

impl TinyUrlHandler {
    pub fn router() -> axum::Router<AppState> {
        Router::new()
        .route("/generate/{url}", get(generate_tiny_url))
        .route("/list-all/{id}", method_router)
        .route("/get/{id}", method_router)
        .route("/deactivate", method_router)
    }
}

async fn generate_tiny_url(State(state): State<AppState>,Path(url): Path<String>, claims: AccessClaims) -> TinyUrlResponse<(StatusCode,Json<URL>)>  {
    let result = Service::new(state).generate(&url, claims).await?;
}
