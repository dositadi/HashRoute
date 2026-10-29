use anyhow::{ Result };
use tracing::{ error, info };

use crate::internal::{
    auth::{
        repository::{ models::UserCreateInput, repo::{ Repo, UserRepo } },
        service::{ error::AuthError, jwt::JWT, models::RefreshClaims, password_hash::Argon },
        usecase::models::{ LoginInput, RegisterInput, RegisterOutput, Token },
    },
    config::jwt_config::JWTConfig,
};

pub struct UseCase<T> where T: Repo {
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

    pub async fn login(&self, input: LoginInput) -> Result<Token, AuthError> {
        let user = self.repo.get_user_by_email(&input.email).await?;

        let _ = Argon.compare(&user.password_hash, &input.password)?;

        let access_resp = self.jwt.access_token(user.id)?;

        let refresh_token = self.jwt.refresh_token(user.id)?;

        let Ok(exp) = time::OffsetDateTime::from_unix_timestamp(
            access_resp.expires_at as i64
        ) else {
            error!("failed to parse time from usize");
            return Err(AuthError::Internal);
        };

        info!(user_id=?user.id, "user login successful");
        Ok(Token {
            access_token: access_resp.access_token,
            refresh_token,
            expires_at: exp,
        })
    }

    pub fn rotate_refresh(&self, claim: RefreshClaims) -> Result<Token, AuthError> {
        let Ok(id) = sqlx::types::Uuid::parse_str(&claim.id) else {
            error!("failed to parse user id");
            return Err(AuthError::InvalidID);
        };

        let access_token = self.jwt.access_token(id)?;

        let refresh_token = self.jwt.refresh_token(id)?;

        let Ok(exp) = time::OffsetDateTime::from_unix_timestamp(
            access_token.expires_at as i64
        ) else {
            error!("failed to parse time from usize");
            return Err(AuthError::Internal);
        };

        info!(user_id=?claim.id,"user token rotated successfully");
        Ok(Token {
            access_token: access_token.access_token,
            refresh_token,
            expires_at: exp,
        })
    }
}
