use anyhow::Result;
use axum::{ Router, http::{ StatusCode }, response::IntoResponse, routing::get };

use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::{ EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt };

use crate::internal::{ AppConfig, config::app_state::AppState };

#[derive(Clone, Debug)]
pub struct Application {
    config: AppConfig,
}

impl Application {
    pub async fn start_server(self) -> Result<()> {
        let listener = TcpListener::bind(self.config.server.to_addr()).await?;

        set_up_global_tracing();

        info!("starting server at {:?}", self.config.server.to_addr());

        let router = Router::new().route("/", get(livez)).with_state(AppState::init());

        axum::serve(listener, router).await?;

        Ok(())
    }
}

impl Default for Application {
    fn default() -> Self {
        Application { config: AppConfig::init() }
    }
}

fn set_up_global_tracing() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        "info,tokio=error".to_string().into()
    });

    tracing_subscriber
        ::registry()
        .with(env_filter)
        .with(
            fmt
                ::layer()
                .json()
                .with_current_span(false)
                .with_line_number(true)
                .with_level(true)
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_file(true)
        )
        .init();
}

async fn livez() -> impl IntoResponse {
    StatusCode::OK
}
