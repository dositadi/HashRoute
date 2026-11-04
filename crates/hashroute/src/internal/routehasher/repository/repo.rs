use sqlx::Result;
use sqlx::postgres::PgDatabaseError;
use tracing::info;

use crate::internal::routehasher::repository::models::{ MappedUrl, URL };
use crate::internal::routehasher::repository::{ errors::URLDBError, models::MappedUrlResp };
use crate::internal::routehasher::queries::query::{
    DEACTIVATE_URL,
    GET_URL_BY_CODE,
    GET_URL_BY_ID,
    INSERT_URL,
    LIST_URLS,
};

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

    pub async fn deactivate_url(&self, id: sqlx::types::Uuid) -> Result<(), URLDBError> {
        let response = sqlx::query(DEACTIVATE_URL).bind(id).execute(&self.db_pool).await;

        match response {
            Ok(pg_res) => {
                if pg_res.rows_affected() > 0 {
                    info!(id=?id,"deactivated an expired url successfully");
                    return Ok(());
                }
                info!(id=?id,"no row affected, no deactivation carried out");
                Err(URLDBError::NoRowAffected)
            }
            Err(why) => {
                match why {
                    sqlx::Error::RowNotFound => {
                        info!(id=?id, "row not found for {:?}", id);
                        Err(URLDBError::NotFound)
                    }
                    sqlx::Error::Io(why) => {
                        info!(error = "IO", "failed to get mapped URL: {:?}", why);
                        Err(URLDBError::IO(why.to_string()))
                    }
                    sqlx::Error::PoolTimedOut => {
                        info!(
                            error = "TIMEOUT",
                            "pool timed out while attempting to get mapped URL"
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

    pub async fn get_url_by_id(&self, id: sqlx::types::Uuid) -> Result<URL, URLDBError> {
        let response = sqlx
            ::query_as::<_, URL>(GET_URL_BY_ID)
            .bind(id)
            .fetch_one(&self.db_pool).await;

        match response {
            Ok(url) => {
                info!(id=?id,"url retrieved successfully");
                Ok(url)
            }
            Err(why) => {
                match why {
                    sqlx::Error::RowNotFound => {
                        info!(id=?id, "row not found for {:?}", id);
                        Err(URLDBError::NotFound)
                    }
                    sqlx::Error::Io(why) => {
                        info!(error = "IO", "failed to get mapped URL: {:?}", why);
                        Err(URLDBError::IO(why.to_string()))
                    }
                    sqlx::Error::PoolTimedOut => {
                        info!(
                            error = "TIMEOUT",
                            "pool timed out while attempting to get mapped URL"
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

    pub async fn get_url_by_code(&self, short_code: String) -> Result<URL, URLDBError> {
        let response = sqlx
            ::query_as::<_, URL>(GET_URL_BY_CODE)
            .bind(&short_code)
            .fetch_one(&self.db_pool).await;

        match response {
            Ok(url) => {
                info!(short_code=?short_code,"url retrieved successfully");
                Ok(url)
            }
            Err(why) => {
                match why {
                    sqlx::Error::RowNotFound => {
                        info!(short_code=?short_code, "row not found for {:?}", short_code);
                        Err(URLDBError::NotFound)
                    }
                    sqlx::Error::Io(why) => {
                        info!(error = "IO", "failed to get mapped URL: {:?}", why);
                        Err(URLDBError::IO(why.to_string()))
                    }
                    sqlx::Error::PoolTimedOut => {
                        info!(
                            error = "TIMEOUT",
                            "pool timed out while attempting to get mapped URL"
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

    pub async fn list_mapped_urls(
        &self,
        user_id: sqlx::types::Uuid
    ) -> Result<Vec<URL>, URLDBError> {
        let response: Result<Vec<URL>, sqlx::Error> = sqlx
            ::query_as(LIST_URLS)
            .bind(user_id)
            .fetch_all(&self.db_pool).await;

        match response {
            Ok(urls) => {
                info!(id=?user_id, "urls retrieved successfully for {:?}", user_id);
                Ok(urls)
            }
            Err(why) => {
                match why {
                    sqlx::Error::RowNotFound => {
                        info!(id=?user_id, "row not found for {}", user_id);
                        Err(URLDBError::NotFound)
                    }
                    sqlx::Error::Io(why) => {
                        info!(error = "IO", "failed to list mapped URL: {:?}", why);
                        Err(URLDBError::IO(why.to_string()))
                    }
                    sqlx::Error::PoolTimedOut => {
                        info!(
                            error = "TIMEOUT",
                            "pool timed out while attempting to list mapped URL"
                        );
                        Err(URLDBError::Timeout)
                    }
                    _ => {
                        info!(
                            error = "INTERNAL",
                            "something wrong happened while listing mapped url"
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
            Ok(resp) => {
                info!(id=?input.user_id, "urls retrieved successfully for {:?}", input.user_id);
                Ok(resp)
            }
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
