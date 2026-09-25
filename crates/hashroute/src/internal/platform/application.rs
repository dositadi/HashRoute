use anyhow::Result;
use axum::{ Router, http::{ StatusCode }, response::IntoResponse, routing::get };
use tokio::net::TcpListener;

use crate::internal::AppConfig;

#[derive(Clone, Debug)]
struct Application {
    config: AppConfig,
}

impl Application {
    pub async fn start_server(self) -> Result<()> {
        let listener = TcpListener::bind(self.config.server.to_addr()).await?;

        let router = Router::new().route("/", get(livez));

        axum::serve(listener, router).await?;

        Ok(())
    }
}

async fn livez() -> impl IntoResponse {
    StatusCode::OK
}
