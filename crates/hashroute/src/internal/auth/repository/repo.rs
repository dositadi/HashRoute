use sqlx::postgres::PgDatabaseError;
use sqlx::{ Result };

use crate::internal::auth::repository::models::{ UserCreateInput, UserCreateResponse };

use crate::internal::auth::queries;

pub enum UserError {
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
    ) -> Result<UserCreateResponse, UserError> {
        let response = sqlx
            ::query_as::<_, UserCreateResponse>(queries::user::CREATEUSER)
            .bind(&user.name)
            .bind(&user.email)
            .bind(&user.password_hash)
            .fetch_one(&self.dbpool).await;

        match response {
            Ok(resp) => Ok(resp),
            Err(why) => {
                match why {
                    sqlx::Error::RowNotFound => Err(UserError::NotFound),
                    sqlx::Error::Database(db_err) => {
                        let e = db_err.downcast_ref::<PgDatabaseError>();

                        match e.constraint() {
                            Some("users_email_exists") => Err(UserError::EmailConflict),
                            _ => Err(UserError::Internal),
                        }
                    }
                    sqlx::Error::PoolTimedOut => Err(UserError::TimeOut),
                    sqlx::Error::Io(_) => Err(UserError::Io("network error occurred".to_owned())),
                    _ => Err(UserError::Internal),
                }
            }
        }
    }

    
}
