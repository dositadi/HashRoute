use anyhow::{ Result };
use tracing::{ error, info };

use crate::internal::{
    auth::{
        repository::{ models::UserCreateInput, repo::{ Repo, UserRepo } },
        service::{ jwt::JWT, models::{ AuthError, JwtAccessTokenResponse }, password_hash::Argon },
        usecase::models::{ LoginInput, RegisterInput, RegisterOutput },
    },
    config::jwt_config::JWTConfig,
};

struct UseCase<T> where T: Repo {
    repo: T,
    jwt: JWT,
}

impl UseCase<UserRepo> {
    pub fn new(pool: sqlx::postgres::PgPool, jwt_cfg: JWTConfig) -> Self {
        let jwt = JWT::new(jwt_cfg);
        Self { repo: UserRepo::new(pool), jwt }
    }

    pub async fn register(&self, user: RegisterInput) -> Result<RegisterOutput, AuthError> {
        let Ok(password_hash) = Argon.hash(&user.password) else {
            error!("failed to hash user password");
            return Err(AuthError::Internal);
        };

        let resp = self.repo.create_user(
            &(UserCreateInput { email: user.email, name: user.name, password_hash })
        ).await?;

        info!(user_id=?resp.id, "user registered successfully");
        Ok(RegisterOutput { created_at: resp.created_at, id: resp.id })
    }

    pub async fn login(&self, input: LoginInput) -> Result<JwtAccessTokenResponse, AuthError> {
        let user = self.repo.get_user_by_email(&input.email).await?;

        let _ = Argon.compare(&user.password_hash, &input.password)?;

        let resp = self.jwt.access_token(user.id)?;

        Ok(resp)
    }
}
