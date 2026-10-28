use axum::{ Form, Router, extract::State, response::IntoResponse, routing::post };
use axum_valid::Valid;

use crate::internal::{ auth::{handler::models::User, usecase::app::UseCase}, config::app_state::AppState };

pub struct AuthHandler;

impl AuthHandler {
    pub fn router(&self) -> axum::Router<AppState> {
        Router::new().route("/register", post(register))
    }
}

async fn register(
    State(state): State<AppState>,
    Valid(Form(user)): Valid<Form<User>>
) -> impl IntoResponse {
    let usecase = UseCase::new(s)
}
