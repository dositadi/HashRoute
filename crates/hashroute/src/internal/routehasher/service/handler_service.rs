use tracing::info;

use crate::internal::{auth::{service::models::AccessClaims}, config::app_state::AppState, routehasher::{domain::{errors::URLDBError, models::{NewURL, STATUS_ACTIVE}}, repository::repo::Repository, service::create_hash::TinyUrl}};
use crate::internal::routehasher::domain::models::URL;

pub struct Service {
    app_state: AppState
}

const DEFAULT_TINY_CODE_SIZE: usize = 5;
const RETRY_LIMIT: usize = 5;

impl Service {
    pub fn new(app_state: AppState) -> Self {
        Service { app_state }
    }

    pub async fn generate(self, url: &str, claims: AccessClaims) -> Result<URL, URLDBError> {
        let db = Repository::new(self.app_state.repository.postgres.db_pool);

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

            let response = db.store_mapped_url( NewURL { 
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
}