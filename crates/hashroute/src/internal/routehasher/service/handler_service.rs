use tracing::info;

use crate::internal::routehasher::domain::models::URL;
use crate::internal::routehasher::service::create_hash::generate_short_code;
use crate::internal::routehasher::service::url_check::check_url;
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

/// The user id comes from a verified token, so a bad one means a bad token.
fn parse_user_id(id: &str) -> Result<sqlx::types::Uuid, URLDBError> {
    sqlx::types::Uuid::parse_str(id).map_err(|why| {
        info!(err=?why, id=?id, "user id in token is invalid");
        URLDBError::InvalidCredential("invalid user id".to_owned())
    })
}

/// The link id comes from the URL path, so a bad one is the client's mistake.
fn parse_link_id(id: &str) -> Result<sqlx::types::Uuid, URLDBError> {
    sqlx::types::Uuid::parse_str(id).map_err(|_| URLDBError::InvalidId)
}

impl Service<Repository> {
    pub fn new(repo: Repository) -> Self {
        Service { repo }
    }
}

impl<T: TinyRepo> Service<T> {
    pub async fn generate(&self, url: &str, user_id: &str) -> Result<URL, URLDBError> {
        let user_id = parse_user_id(user_id)?;
        let url = check_url(url)?;

        for attempt in 0..MAX_ATTEMPTS {
            let response = self
                .repo
                .store_mapped_url(NewURL {
                    user_id,
                    short_code: generate_short_code(DEFAULT_TINY_CODE_SIZE + attempt),
                    original_url: url.clone(),
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
        let user_id = parse_user_id(user_id)?;

        self.repo.list_mapped_urls(user_id).await
    }

    pub async fn deactivate_url(&self, id: &str, user_id: &str) -> Result<(), URLDBError> {
        let id = parse_link_id(id)?;
        let user_id = parse_user_id(user_id)?;

        self.repo.deactivate_url(id, user_id).await
    }

    pub async fn get_url_by_id(&self, id: &str, user_id: &str) -> Result<URL, URLDBError> {
        let id = parse_link_id(id)?;
        let user_id = parse_user_id(user_id)?;

        self.repo.get_url_by_id(id, user_id).await
    }

    pub async fn get_url_by_code(&self, short_code: String) -> Result<URL, URLDBError> {
        self.repo.get_url_by_code(short_code).await
    }
}

#[cfg(test)]
mod test {
    use std::sync::Mutex;

    use super::*;

    const USER: &str = "6f1c2b8e-9a4d-4c3e-8f7a-1b2c3d4e5f60";

    /// Collides on the first `collisions` inserts, then succeeds.
    struct FakeRepo {
        collisions: usize,
        inserted: Mutex<Vec<NewURL>>,
    }

    impl FakeRepo {
        fn new(collisions: usize) -> Self {
            Self { collisions, inserted: Mutex::new(Vec::new()) }
        }
    }

    impl TinyRepo for FakeRepo {
        async fn get_url_by_code(&self, _: String) -> Result<URL, URLDBError> {
            Err(URLDBError::NotFound)
        }
        async fn deactivate_url(&self, _: Uuid, _: Uuid) -> Result<(), URLDBError> {
            Ok(())
        }
        async fn get_url_by_id(&self, _: Uuid, _: Uuid) -> Result<URL, URLDBError> {
            Err(URLDBError::NotFound)
        }
        async fn list_mapped_urls(&self, _: Uuid) -> Result<Vec<URL>, URLDBError> {
            Ok(Vec::new())
        }
        async fn store_mapped_url(&self, input: NewURL) -> Result<URL, URLDBError> {
            let mut inserted = self.inserted.lock().unwrap();
            inserted.push(input.clone());
            if inserted.len() <= self.collisions {
                return Err(URLDBError::ShortCodeExists);
            }
            Ok(URL {
                id: Uuid::new_v4(),
                user_id: input.user_id,
                short_code: input.short_code,
                original_url: input.original_url,
                status: input.status,
                created_at: time::OffsetDateTime::now_utc(),
                expiration_date: None,
            })
        }
    }

    use sqlx::types::Uuid;

    #[tokio::test]
    async fn rejects_unsafe_url_before_touching_the_db() {
        let service = Service { repo: FakeRepo::new(0) };
        let err = service.generate("javascript:alert(1)", USER).await.unwrap_err();
        assert!(matches!(err, URLDBError::InvalidUrl(_)));
        assert!(service.repo.inserted.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn retries_with_longer_codes_after_collisions() {
        let service = Service { repo: FakeRepo::new(2) };
        let url = service.generate("https://example.com", USER).await.unwrap();

        let lens: Vec<usize> =
            service.repo.inserted.lock().unwrap().iter().map(|n| n.short_code.len()).collect();
        assert_eq!(lens, vec![6, 7, 8]);
        assert_eq!(url.original_url, "https://example.com/");
    }

    #[tokio::test]
    async fn gives_up_after_max_attempts() {
        let service = Service { repo: FakeRepo::new(MAX_ATTEMPTS) };
        let err = service.generate("https://example.com", USER).await.unwrap_err();
        assert!(matches!(err, URLDBError::Internal));
    }

    #[tokio::test]
    async fn bad_link_id_is_a_client_error() {
        let service = Service { repo: FakeRepo::new(0) };
        let err = service.get_url_by_id("not-a-uuid", USER).await.unwrap_err();
        assert!(matches!(err, URLDBError::InvalidId));
    }
}
