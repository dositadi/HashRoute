use sqlx::Result;
use sqlx::postgres::PgDatabaseError;
use tracing::info;

use crate::internal::routehasher::repository::models::{ MappedUrl, URL };
use crate::internal::routehasher::repository::{ errors::URLDBError, models::MappedUrlResp };
use crate::internal::routehasher::queries::query::{ INSERT_URL, LIST_URL };

type Constraint = &'static str;

const STATUS_CONST: Constraint = "status_contraint";
const UNIQUE_SHORT_CODE: Constraint = "unique_shortcode_contraint";

#[derive(Debug, Clone)]
pub struct Repository {
    db_pool: sqlx::postgres::PgPool,
}

impl Repository {
    pub fn new(db_pool: sqlx::postgres::PgPool) -> Self {
        Self { db_pool }
    }

    pub async fn get_urls(&self, id: sqlx::types::Uuid) -> Result<URL, URLDBError> {
        let response = sqlx::query_as::<_, URL>(sql)
    }

    pub async fn list_mapped_urls(
        &self,
        user_id: sqlx::types::Uuid
    ) -> Result<Vec<URL>, URLDBError> {
        let urls: Result<Vec<URL>, sqlx::Error> = sqlx
            ::query_as(LIST_URL)
            .bind(user_id)
            .fetch_all(&self.db_pool).await;

        match urls {
            Ok(response) => Ok(response),
            Err(why) => {
                match why {
                    sqlx::Error::RowNotFound => {
                        info!(id=?user_id, "row not found for {}", user_id);
                        Err(URLDBError::NotFound)
                    }
                    sqlx::Error::Io(why) => {
                        info!(error = "IO", "failed to store mapped URL: {:?}", why);
                        Err(URLDBError::IO(why.to_string()))
                    }
                    sqlx::Error::PoolTimedOut => {
                        info!(
                            error = "TIMEOUT",
                            "pool timed out while attempting to store mapped URL"
                        );
                        Err(URLDBError::Timeout)
                    }
                    _ => {
                        info!(
                            error = "INTERNAL",
                            "something wrong happened while storing mapped url"
                        );
                        Err(URLDBError::Internal)
                    }
                }
            }
        }
    }

    pub async fn store_mapped_url(&self, input: MappedUrl) -> Result<MappedUrlResp, URLDBError> {
        let result = sqlx
            ::query_as::<_, MappedUrlResp>(INSERT_URL)
            .bind(input.user_id)
            .bind(input.short_code)
            .bind(input.original_url)
            .bind(input.status)
            .fetch_one(&self.db_pool).await;

        match result {
            Ok(resp) => { Ok(resp) }
            Err(why) => {
                match why {
                    sqlx::Error::Database(db_err) => {
                        let pg_err = db_err.downcast_ref::<PgDatabaseError>();

                        let Some(cons) = pg_err.constraint() else {
                            return Err(URLDBError::Internal);
                        };

                        if cons.eq(STATUS_CONST) {
                            info!(
                                error = "status constraint",
                                "invalid status constriant why attempting to store mapped url"
                            );
                            return Err(URLDBError::InvalidStatus);
                        } else if cons.eq(UNIQUE_SHORT_CODE) {
                            info!(
                                error = "short code constraint",
                                "mapped url short code exists already"
                            );
                            return Err(URLDBError::ShortCodeExists);
                        }

                        info!(error = "internal", "something wrong happened");

                        Err(URLDBError::Internal)
                    }
                    sqlx::Error::Io(why) => {
                        info!(error = "IO", "failed to store mapped URL: {:?}", why);
                        Err(URLDBError::IO(why.to_string()))
                    }
                    sqlx::Error::PoolTimedOut => {
                        info!(
                            error = "TIMEOUT",
                            "pool timed out while attempting to store mapped URL"
                        );
                        Err(URLDBError::Timeout)
                    }
                    _ => {
                        info!(
                            error = "INTERNAL",
                            "something wrong happened while storing mapped url"
                        );
                        Err(URLDBError::Internal)
                    }
                }
            }
        }
    }
}
