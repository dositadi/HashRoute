use tracing::info;

use crate::internal::routehasher::domain::models::URL;
use crate::internal::routehasher::service::create_hash::generate_short_code;
use crate::internal::routehasher::{
    domain::{
        errors::URLDBError,
        models::{NewURL, STATUS_ACTIVE},
    },
    repository::repo::Repository,
};

pub trait TinyRepo {
    fn get_url_by_code(
        &self,
        short_code: String,
    ) -> impl std::future::Future<Output = Result<URL, URLDBError>> + Send;
    fn deactivate_url(
        &self,
        id: sqlx::types::Uuid,
        user_id: sqlx::types::Uuid,
    ) -> impl std::future::Future<Output = Result<(), URLDBError>> + Send;
    fn get_url_by_id(
        &self,
        id: sqlx::types::Uuid,
        user_id: sqlx::types::Uuid,
    ) -> impl std::future::Future<Output = Result<URL, URLDBError>> + Send;
    fn list_mapped_urls(
        &self,
        user_id: sqlx::types::Uuid,
    ) -> impl std::future::Future<Output = Result<Vec<URL>, URLDBError>> + Send;
    fn store_mapped_url(
        &self,
        input: NewURL,
    ) -> impl std::future::Future<Output = Result<URL, URLDBError>> + Send;
}

pub struct Service<T>
where
    T: TinyRepo,
{
    repo: T,
}

const DEFAULT_TINY_CODE_SIZE: usize = 6;
/// How many times to try a fresh code after a collision. Each retry is one character longer.
const MAX_ATTEMPTS: usize = 5;

fn parse_uuid(id: &str, what: &str) -> Result<sqlx::types::Uuid, URLDBError> {
    sqlx::types::Uuid::parse_str(id).map_err(|why| {
        info!(err=?why, id=?id, "{what} is invalid");
        URLDBError::InvalidCredential(format!("invalid {what}"))
    })
}

impl Service<Repository> {
    pub fn new(repo: Repository) -> Self {
        Service { repo }
    }
}

impl<T: TinyRepo> Service<T> {
    pub async fn generate(&self, url: &str, user_id: &str) -> Result<URL, URLDBError> {
        let user_id = parse_uuid(user_id, "user id")?;

        for attempt in 0..MAX_ATTEMPTS {
            let response = self
                .repo
                .store_mapped_url(NewURL {
                    user_id,
                    short_code: generate_short_code(DEFAULT_TINY_CODE_SIZE + attempt),
                    original_url: url.to_owned(),
                    status: STATUS_ACTIVE.to_owned(),
                })
                .await;

            match response {
                Err(URLDBError::ShortCodeExists) => {
                    info!(attempt, "short code collision, retrying");
                }
                other => return other,
            }
        }

        Err(URLDBError::Internal)
    }

    pub async fn list_urls(&self, user_id: &str) -> Result<Vec<URL>, URLDBError> {
        let user_id = parse_uuid(user_id, "user id")?;

        self.repo.list_mapped_urls(user_id).await
    }

    pub async fn deactivate_url(&self, id: &str, user_id: &str) -> Result<(), URLDBError> {
        let id = parse_uuid(id, "url id")?;
        let user_id = parse_uuid(user_id, "user id")?;

        self.repo.deactivate_url(id, user_id).await
    }

    pub async fn get_url_by_id(&self, id: &str, user_id: &str) -> Result<URL, URLDBError> {
        let id = parse_uuid(id, "url id")?;
        let user_id = parse_uuid(user_id, "user id")?;

        self.repo.get_url_by_id(id, user_id).await
    }

    pub async fn get_url_by_code(&self, short_code: String) -> Result<URL, URLDBError> {
        self.repo.get_url_by_code(short_code).await
    }
}
