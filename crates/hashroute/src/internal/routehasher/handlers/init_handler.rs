use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::{Redirect, Response},
    routing::{get, post},
};

use crate::internal::{
    auth::service::models::AccessClaims,
    config::app_state::AppState,
    platform::api::{self, ValidJson},
    routehasher::{
        domain::{
            errors::URLDBError,
            models::{CreateLinkPayload, LinkView},
        },
        repository::repo::Repository,
        service::handler_service::Service,
    },
};

type LinkResponse = Result<Response, URLDBError>;

fn service(state: &AppState) -> Service<Repository> {
    Service::new(Repository::new(state.repository.postgres.db_pool.clone()))
}

pub struct TinyUrlHandler;

impl TinyUrlHandler {
    /// Authenticated link API, nested under `/tiny/v1/links`.
    pub fn router() -> axum::Router<AppState> {
        Router::new()
            .route("/", get(list_links).post(create_link))
            .route("/{id}", get(get_link))
            .route("/{id}/deactivate", post(deactivate_link))
    }

    /// Public redirect route, mounted at the root: `GET /{code}`.
    pub fn redirect_router() -> axum::Router<AppState> {
        Router::new().route("/{code}", get(redirect))
    }
}

async fn redirect(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> Result<Redirect, URLDBError> {
    let url = service(&state).get_url_by_code(code).await?;

    Ok(Redirect::temporary(&url.original_url))
}

/// `POST /tiny/v1/links` with `{ "url": "https://..." }` → 201 with the new link.
async fn create_link(
    State(state): State<AppState>,
    claims: AccessClaims,
    ValidJson(payload): ValidJson<CreateLinkPayload>,
) -> LinkResponse {
    let url = service(&state).generate(&payload.url, &claims.sub).await?;

    Ok(api::ok(StatusCode::CREATED, LinkView::new(url, &state.public_base_url)))
}

/// `GET /tiny/v1/links` → the caller's active links, newest first.
async fn list_links(State(state): State<AppState>, claims: AccessClaims) -> LinkResponse {
    let links: Vec<LinkView> = service(&state)
        .list_urls(&claims.sub)
        .await?
        .into_iter()
        .map(|url| LinkView::new(url, &state.public_base_url))
        .collect();

    Ok(api::ok(StatusCode::OK, links))
}

/// `GET /tiny/v1/links/{id}` → one of the caller's links.
async fn get_link(
    State(state): State<AppState>,
    Path(id): Path<String>,
    claims: AccessClaims,
) -> LinkResponse {
    let url = service(&state).get_url_by_id(&id, &claims.sub).await?;

    Ok(api::ok(StatusCode::OK, LinkView::new(url, &state.public_base_url)))
}

/// `POST /tiny/v1/links/{id}/deactivate` → 204; the short code stops redirecting.
async fn deactivate_link(
    State(state): State<AppState>,
    Path(id): Path<String>,
    claims: AccessClaims,
) -> Result<StatusCode, URLDBError> {
    service(&state).deactivate_url(&id, &claims.sub).await?;

    Ok(StatusCode::NO_CONTENT)
}
