use sqlx::postgres::PgDatabaseError;
use sqlx::{ Result };
use tracing::info;

use crate::internal::auth::repository::error::DBError;
use crate::internal::auth::repository::models::{ User, UserCreateInput, UserCreateResponse };

use crate::internal::auth::queries;

pub trait Repo {
    fn create_user(&self, user: &UserCreateInput) -> impl std::future::Future<Output = Result<UserCreateResponse, DBError>> + Send;
    fn get_user_by_id(&self, id: sqlx::types::Uuid) -> impl std::future::Future<Output = Result<User, DBError>> + Send;
    fn get_user_by_email(&self, email: &str) -> impl std::future::Future<Output = Result<User, DBError>> + Send;
}

type ErrMessage = &'static str;

const CREATE_FAIL_MESSAGE: ErrMessage = "failed to create user";
const FETCH_USER_FAIL_MESSAGE: ErrMessage = "failed to fetch user";

#[derive(Debug, Clone)]
pub struct UserRepo {
    dbpool: sqlx::postgres::PgPool,
}

impl UserRepo {
    pub fn new(pool: sqlx::postgres::PgPool) -> Self {
        Self { dbpool: pool }
    }
}

impl Repo for UserRepo {
    async fn create_user(&self, user: &UserCreateInput) -> Result<UserCreateResponse, DBError> {
        let response = sqlx
            ::query_as::<_, UserCreateResponse>(queries::user::CREATEUSER)
            .bind(&user.name)
            .bind(&user.email)
            .bind(&user.password_hash)
            .fetch_one(&self.dbpool).await;

        match response {
            Ok(resp) => {
                info!(user.name, id=?&resp.id, created_at=?&resp.created_at, "user created successfully");
                Ok(resp)
            }
            Err(why) => {
                match why {
                    sqlx::Error::RowNotFound => Err(DBError::NotFound),
                    sqlx::Error::Database(db_err) => {
                        let e = db_err.downcast_ref::<PgDatabaseError>();
                        info!(user.name, CREATE_FAIL_MESSAGE);
                        match e.constraint() {
                            Some("users_email_exists") => Err(DBError::EmailConflict),
                            _ => Err(DBError::Internal),
                        }
                    }
                    sqlx::Error::PoolTimedOut => {
                        info!(user.name, CREATE_FAIL_MESSAGE);
                        Err(DBError::TimeOut)
                    }
                    sqlx::Error::Io(_) => {
                        info!(user.name, CREATE_FAIL_MESSAGE);
                        Err(DBError::Io("network error occurred".to_owned()))
                    }
                    _ => Err(DBError::Internal),
                }
            }
        }
    }

    async fn get_user_by_id(&self, id: sqlx::types::Uuid) -> Result<User, DBError> {
        let response = sqlx
            ::query_as::<_, User>(queries::user::GETUSERBYID)
            .bind(id)
            .fetch_one(&self.dbpool).await;

        match response {
            Ok(resp) => {
                info!(user_id=?&resp.id,"fetched user successfully");
                Ok(resp)
            }
            Err(why) => {
                match why {
                    sqlx::Error::RowNotFound => {
                        info!(FETCH_USER_FAIL_MESSAGE);
                        Err(DBError::NotFound)
                    }
                    sqlx::Error::PoolTimedOut => {
                        info!(FETCH_USER_FAIL_MESSAGE);
                        Err(DBError::TimeOut)
                    }
                    _ => {
                        info!(FETCH_USER_FAIL_MESSAGE);
                        Err(DBError::Internal)
                    }
                }
            }
        }
    }

    async fn get_user_by_email(&self, email: &str) -> Result<User, DBError> {
        let response = sqlx
            ::query_as::<_, User>(queries::user::GETUSERBYEMAIL)
            .bind(email)
            .fetch_one(&self.dbpool).await;

        match response {
            Ok(resp) => {
                info!(user_id=?&resp.id, "fetched user successfully");
                Ok(resp)
            }
            Err(why) => {
                match why {
                    sqlx::Error::RowNotFound => {
                        info!(FETCH_USER_FAIL_MESSAGE);
                        Err(DBError::NotFound)
                    }
                    sqlx::Error::PoolTimedOut => {
                        info!(FETCH_USER_FAIL_MESSAGE);
                        Err(DBError::TimeOut)
                    }
                    _ => {
                        info!(FETCH_USER_FAIL_MESSAGE);
                        Err(DBError::Internal)
                    }
                }
            }
        }
    }
}
