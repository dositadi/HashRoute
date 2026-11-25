use anyhow::Result;
use axum::{Router, http::StatusCode, response::Response, routing::get};

use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use crate::internal::{
    AppConfig,
    auth::handler::handlers::AuthHandler,
    config::{app_state::AppState, cors_config::CORSConfig},
    platform::{
        api::{self, method_not_allowed_fallback, not_found_fallback},
        app::cors::cors,
    },
    routehasher::handlers::init_handler::TinyUrlHandler,
};

#[derive(Clone, Debug)]
pub struct Application {
    config: AppConfig,
}

impl Application {
    pub async fn start_server(self) -> Result<()> {
        set_up_global_tracing();

        let cfg = self.config;

        let listener = TcpListener::bind(cfg.server.to_addr()).await?;

        let state = AppState::init(&cfg).await;

        state.repository.postgres.migrate().await?;

        info!("starting server at {:?}", cfg.server.to_addr());

        axum::serve(listener, build_router(state, &cfg.cors)).await?;

        Ok(())
    }
}

impl Default for Application {
    fn default() -> Self {
        Application { config: AppConfig::init() }
    }
}

fn set_up_global_tracing() {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,tokio=error".to_string().into());

    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            fmt::layer()
                .json()
                .with_current_span(false)
                .with_line_number(true)
                .with_level(true)
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_file(true),
        )
        .init();
}

/// Every route, with JSON fallbacks for unknown paths and wrong methods.
pub fn build_router(state: AppState, cors_cfg: &CORSConfig) -> Router {
    Router::new()
        .route("/", get(livez))
        .nest("/tiny/v1/auth", AuthHandler.router())
        .nest("/tiny/v1/links", TinyUrlHandler::router())
        .merge(TinyUrlHandler::redirect_router())
        .fallback(not_found_fallback)
        .method_not_allowed_fallback(method_not_allowed_fallback)
        .layer(cors(cors_cfg))
        .with_state(state)
}

async fn livez() -> Response {
    api::ok(StatusCode::OK, serde_json::json!({ "status": "ok" }))
}
