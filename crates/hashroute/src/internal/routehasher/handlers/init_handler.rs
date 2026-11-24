use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
    routing::{get, post},
};
use serde::Serialize;
use serde_json::json;

use crate::internal::{
    auth::service::models::AccessClaims,
    config::app_state::AppState,
    routehasher::{
        domain::{
            errors::URLDBError,
            models::{IdQuery, URLQuery},
        },
        repository::repo::Repository,
        service::handler_service::Service,
    },
};

type TinyUrlResponse<T> = Result<T, URLDBError>;

fn success_response<T>(resp: T, status_code: StatusCode) -> (StatusCode, impl IntoResponse)
where
    T: Serialize + Send,
{
    let code = status_code.as_u16();
    let body = Json(json!(
        {
            "code": code,
            "body": resp
        }
    ));

    (status_code, body)
}

fn service(state: AppState) -> Service<Repository> {
    Service::new(Repository::new(state.repository.postgres.db_pool))
}

pub struct TinyUrlHandler;

impl TinyUrlHandler {
    /// Authenticated management API, nested under `/tiny/v1/tiny-url`.
    pub fn router() -> axum::Router<AppState> {
        Router::new()
            .route("/generate", post(generate_tiny_url))
            .route("/list-all", get(list_urls))
            .route("/get", get(get_url))
            .route("/deactivate", post(deactivate))
    }

    /// Public redirect route, mounted at the root: `GET /{code}`.
    pub fn redirect_router() -> axum::Router<AppState> {
        Router::new().route("/{code}", get(redirect))
    }
}

async fn redirect(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> TinyUrlResponse<Redirect> {
    let url = service(state).get_url_by_code(code).await?;

    Ok(Redirect::temporary(&url.original_url))
}

async fn deactivate(
    State(state): State<AppState>,
    Query(query): Query<IdQuery>,
    claims: AccessClaims,
) -> TinyUrlResponse<(StatusCode, impl IntoResponse)> {
    service(state).deactivate_url(&query.id, &claims.sub).await?;

    Ok(success_response("success".to_owned(), StatusCode::OK))
}

async fn get_url(
    State(state): State<AppState>,
    Query(query): Query<IdQuery>,
    claims: AccessClaims,
) -> TinyUrlResponse<(StatusCode, impl IntoResponse)> {
    let url = service(state).get_url_by_id(&query.id, &claims.sub).await?;

    Ok(success_response(url, StatusCode::OK))
}

async fn generate_tiny_url(
    State(state): State<AppState>,
    Query(url): Query<URLQuery>,
    claims: AccessClaims,
) -> TinyUrlResponse<(StatusCode, impl IntoResponse)> {
    let result = service(state).generate(&url.url, &claims.sub).await?;

    Ok(success_response(result, StatusCode::CREATED))
}

async fn list_urls(
    State(state): State<AppState>,
    claims: AccessClaims,
) -> TinyUrlResponse<(StatusCode, impl IntoResponse)> {
    let urls = service(state).list_urls(&claims.sub).await?;

    Ok(success_response(urls, StatusCode::OK))
}
