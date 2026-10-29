use axum::{ Form, Json, Router, extract::State, routing::post };
use axum_valid::Valid;
use tracing::info;

use crate::internal::{
    auth::{
        handler::models::{ LoginPayload, RegisterPayload },
        service::error::AuthError,
        usecase::{ app::UseCase, models::{ LoginInput, RegisterInput, RegisterOutput, Token } },
    },
    config::app_state::AppState,
};

pub struct AuthHandler;

impl AuthHandler {
    pub fn router(&self) -> axum::Router<AppState> {
        Router::new().route("/register", post(register)).route("/login", post(login))
    }
}

async fn register(
    State(state): State<AppState>,
    Valid(Form(user)): Valid<Form<RegisterPayload>>
) -> Result<Json<RegisterOutput>, AuthError> {
    let usecase = UseCase::new(state.repository.postgres.db_pool, state.jwt_secret);

    let resp = usecase.register(RegisterInput {
        email: user.email,
        name: user.name,
        password: user.password,
    }).await?;

    info!("registered a user successsfully");
    Ok(Json(resp))
}

async fn login(
    State(state): State<AppState>,
    Valid(Form(user)): Valid<Form<LoginPayload>>
) -> Result<Json<Token>, AuthError> {
    let usecase = UseCase::new(state.repository.postgres.db_pool, state.jwt_secret);

    let resp = usecase.login(LoginInput {
        email: user.email.clone(),
        password: user.password,
    }).await?;

    info!(user=?user.email,"user login successful");
    Ok(Json(resp))
}
