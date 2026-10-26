use anyhow::Ok;
use sqlx::postgres::PgDatabaseError;
use sqlx::{ Result };
use tracing::info;

use crate::internal::auth::repository::models::{ User, UserCreateInput, UserCreateResponse };

use crate::internal::auth::queries;

pub enum DBError {
    NotFound,
    EmailConflict,
    Internal,
    TimeOut,
    Io(String),
}

#[derive(Debug, Clone)]
pub struct Repository {
    pub dbpool: sqlx::postgres::PgPool,
}

impl Repository {
    pub fn new(pool: sqlx::postgres::PgPool) -> Self {
        Self { dbpool: pool }
    }

    pub async fn create_user(
        &self,
        user: &UserCreateInput
    ) -> Result<UserCreateResponse, DBError> {
        let response = sqlx
            ::query_as::<_, UserCreateResponse>(queries::user::CREATEUSER)
            .bind(&user.name)
            .bind(&user.email)
            .bind(&user.password_hash)
            .fetch_one(&self.dbpool).await;

        match response {
            Ok(resp) => {
                info!(user.name, id=?resp.id, created_at=?resp.created_at, "user created successfully");
                Ok(resp)
            }
            Err(why) => {
                match why {
                    sqlx::Error::RowNotFound => Err(DBError::NotFound),
                    sqlx::Error::Database(db_err) => {
                        let e = db_err.downcast_ref::<PgDatabaseError>();
                        info!(user.name, "failed to create user");
                        match e.constraint() {
                            Some("users_email_exists") => Err(DBError::EmailConflict),
                            _ => Err(DBError::Internal),
                        }
                    }
                    sqlx::Error::PoolTimedOut => {
                        info!(user.name, "failed to create user");
                        Err(DBError::TimeOut)
                    }
                    sqlx::Error::Io(_) => {
                        info!(user.name, "failed to create user");
                        Err(DBError::Io("network error occurred".to_owned()))
                    }
                    _ => Err(DBError::Internal),
                }
            }
        }
    }

    pub async fn get_user_by_id(&self, id: sqlx::types::Uuid) -> Result<User, DBError> {
        let user = sqlx::query_as::<_, User>(sql)
            .bind(id)
            .fetch_one(&self.dbpool)
            .await;

        user
    }
}
