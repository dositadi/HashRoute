use axum::{ Router, extract::State, http::StatusCode, response::Response, routing::{ get, post } };
use tracing::info;

use crate::internal::{
    auth::{
        handler::models::{ LoginPayload, RegisterPayload },
        service::{ error::AuthError, models::{ AccessClaims, RefreshClaims } },
        usecase::{ app::UseCase, models::{ LoginInput, RegisterInput } },
    },
    config::app_state::AppState,
    platform::api::{ self, ValidJson },
};

pub struct AuthHandler;

type AppResponse = Result<Response, AuthError>;

impl AuthHandler {
    pub fn router(&self) -> axum::Router<AppState> {
        Router::new()
            .route("/register", post(register))
            .route("/login", post(login))
            .route("/rotate-refresh", post(rotate_refresh))
            .route("/me", get(me))
    }
}

async fn register(
    State(state): State<AppState>,
    ValidJson(user): ValidJson<RegisterPayload>
) -> AppResponse {
    let usecase = UseCase::new(state.repository.postgres.db_pool, state.jwt_secret);

    let resp = usecase.register(RegisterInput {
        email: user.email,
        name: user.name,
        password: user.password,
    }).await?;

    info!("registered a user successsfully");
    Ok(api::ok(StatusCode::CREATED, resp))
}

async fn login(
    State(state): State<AppState>,
    ValidJson(user): ValidJson<LoginPayload>
) -> AppResponse {
    let usecase = UseCase::new(state.repository.postgres.db_pool, state.jwt_secret);

    let resp = usecase.login(LoginInput {
        email: user.email.clone(),
        password: user.password,
    }).await?;

    info!(user=?user.email,"user login successful");
    Ok(api::ok(StatusCode::OK, resp))
}

async fn rotate_refresh(
    State(state): State<AppState>,
    claims: RefreshClaims
) -> AppResponse {
    let usecase = UseCase::new(state.repository.postgres.db_pool, state.jwt_secret);

    let resp = usecase.rotate_refresh(claims)?;

    Ok(api::ok(StatusCode::OK, resp))
}

/// `GET /tiny/v1/auth/me` → the logged-in user's profile.
async fn me(State(state): State<AppState>, claims: AccessClaims) -> AppResponse {
    let usecase = UseCase::new(state.repository.postgres.db_pool, state.jwt_secret);

    let resp = usecase.me(&claims.sub).await?;

    Ok(api::ok(StatusCode::OK, resp))
}
