use tracing::info;

use crate::internal::{auth::service::models::AccessClaims, routehasher::{domain::{errors::URLDBError, models::{NewURL, STATUS_ACTIVE}}, repository::repo::{Repository}, service::create_hash::TinyUrl}};
use crate::internal::routehasher::domain::models::URL;

pub trait TinyRepo {
    fn get_url_by_code(&self, short_code: String) -> impl std::future::Future<Output = Result<URL, URLDBError>> + Send;
    fn deactivate_url(&self, id: sqlx::types::Uuid) -> impl std::future::Future<Output = Result<(), URLDBError>> + Send;
    fn get_url_by_id(&self, id: sqlx::types::Uuid) -> impl std::future::Future<Output = Result<URL, URLDBError>> + Send;
    fn list_mapped_urls(
        &self,
        user_id: sqlx::types::Uuid
    ) -> impl std::future::Future<Output = Result<Vec<URL>, URLDBError>> + Send;
    fn store_mapped_url(&self, input: NewURL) -> impl std::future::Future<Output = Result<URL, URLDBError>> + Send;
}

pub struct Service<T> where T: TinyRepo {
    repo: T,
}

const DEFAULT_TINY_CODE_SIZE: usize = 5;
const RETRY_LIMIT: usize = 5;

impl Service<Repository> {
    pub fn new(repo: Repository) -> Self {
        Service { repo }
    }

    pub async fn generate(self, url: &str, claims: AccessClaims) -> Result<URL, URLDBError> {
        let user_id = match sqlx::types::Uuid::parse_str(&claims.sub){
            Ok(id) => id,
            Err(why) => {
                info!(err=?why,user_id=?&claims.sub,"user id is invalid");
                return Err(URLDBError::InvalidCredential("invalid id".to_owned()));
            }
        };

        let mut size = DEFAULT_TINY_CODE_SIZE;

        loop {
            let code = TinyUrl::new(&url, size).get_tiny_url();

            let response = self.repo.store_mapped_url( NewURL { 
                user_id: user_id, 
                short_code: code, 
                original_url: url.to_owned(), 
                status: STATUS_ACTIVE.to_owned() 
            }).await;

            match response {
                Ok(resp)=> return Ok(resp),
                Err(why) => {
                    match why {
                        URLDBError::ShortCodeExists => {
                            if size.eq(&RETRY_LIMIT) {
                                return Err(URLDBError::Internal);
                            }
                            size = size + 1;
                            continue
                        },
                        _ => {
                            return Err(why)
                        }
                    }
                }
            }
        }
    }

    pub async fn list_urls(self, id: String) -> Result<Vec<URL>, URLDBError> {
        let user_id = match sqlx::types::Uuid::parse_str(&id){
            Ok(id) => id,
            Err(why) => {
                info!(err=?why,user_id=?&id,"user id is invalid");
                return Err(URLDBError::InvalidCredential("invalid id".to_owned()));
            }
        };

        let urls = self.repo.list_mapped_urls(user_id).await?;

        Ok(urls)
    }

    pub async fn deactivate_url(self, id: String) -> Result<(), URLDBError> {
        let id = match sqlx::types::Uuid::parse_str(&id){
            Ok(id) => id,
            Err(why) => {
                info!(err=?why,user_id=?&id,"user id is invalid");
                return Err(URLDBError::InvalidCredential("invalid id".to_owned()));
            }
        };

        let _ = self.repo.deactivate_url(id).await?;

        Ok(())
    }

    pub async fn get_url_by_id(&self, id: String) -> Result<URL, URLDBError> {
        let id = match sqlx::types::Uuid::parse_str(&id){
            Ok(id) => id,
            Err(why) => {
                info!(err=?why,user_id=?&id,"user id is invalid");
                return Err(URLDBError::InvalidCredential("invalid id".to_owned()));
            }
        };
        
        let  url = self.repo.get_url_by_id(id).await?;

        Ok(url)
    }
    
}
