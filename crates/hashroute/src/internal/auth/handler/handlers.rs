use axum::{ Form, Json, Router, extract::State, routing::post };
use axum_valid::Valid;

use crate::internal::{
    auth::{
        handler::models::User,
        service::error::AuthError,
        usecase::{ app::UseCase, models::{ RegisterInput, RegisterOutput } },
    },
    config::app_state::AppState,
};

pub struct AuthHandler;

impl AuthHandler {
    pub fn router(&self) -> axum::Router<AppState> {
        Router::new().route("/register", post(register))
    }
}

async fn register(
    State(state): State<AppState>,
    Valid(Form(user)): Valid<Form<User>>
) -> Result<Json<RegisterOutput>, AuthError> {
    let usecase = UseCase::new(state.repository.postgres.db_pool, state.jwt_secret);

    let resp = match
        usecase.register(RegisterInput {
            email: user.email,
            name: user.name,
            password: user.password,
        }).await
    {
        Err(why) => {
            return Err(why);
        }
        Ok(value) => value,
    };

    Ok(Json(resp))
}
