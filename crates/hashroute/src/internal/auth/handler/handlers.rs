use axum::{
    Form,
    Json,
    Router,
    extract::State,
    http::StatusCode,
    routing::post,
};
use axum_valid::Valid;
use tracing::info;

use crate::internal::{
    auth::{
        handler::models::{ LoginPayload, RegisterPayload },
        service::{ error::AuthError, models::RefreshClaims },
        usecase::{ app::UseCase, models::{ LoginInput, RegisterInput, RegisterOutput, Token } },
    },
    config::app_state::AppState,
};

pub struct AuthHandler;

type AppResponse<T> = Result<T, AuthError>;

impl AuthHandler {
    pub fn router(&self) -> axum::Router<AppState> {
        Router::new()
            .route("/register", post(register))
            .route("/login", post(login))
            .route("/rotate-refresh", post(rotate_refresh))
    }
}

async fn register(
    State(state): State<AppState>,
    Valid(Form(user)): Valid<Form<RegisterPayload>>
) -> AppResponse<(StatusCode, Json<RegisterOutput>)> {
    let usecase = UseCase::new(state.repository.postgres.db_pool, state.jwt_secret);

    let resp = usecase.register(RegisterInput {
        email: user.email,
        name: user.name,
        password: user.password,
    }).await?;

    info!("registered a user successsfully");
    Ok((StatusCode::CREATED, Json(resp)))
}

async fn login(
    State(state): State<AppState>,
    Valid(Form(user)): Valid<Form<LoginPayload>>
) -> AppResponse<(StatusCode, Json<Token>)> {
    let usecase = UseCase::new(state.repository.postgres.db_pool, state.jwt_secret);

    let resp = usecase.login(LoginInput {
        email: user.email.clone(),
        password: user.password,
    }).await?;

    info!(user=?user.email,"user login successful");
    Ok((StatusCode::OK, Json(resp)))
}

async fn rotate_refresh(
    State(state): State<AppState>,
    claims: RefreshClaims
) -> AppResponse<(StatusCode, Json<Token>)> {
    let usecase = UseCase::new(state.repository.postgres.db_pool, state.jwt_secret);

    let resp = usecase.rotate_refresh(claims)?;

    Ok((StatusCode::OK, Json(resp)))
}
