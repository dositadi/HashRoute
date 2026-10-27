use std::{str::{self, pattern::Pattern}, thread::AccessError};

use anyhow::{ Result };
use jsonwebtoken::{ Header, decode, encode };
use tracing::error;

use crate::internal::{
    auth::service::models::{ AuthError, Claims, Keys },
    config::jwt_config::JWTConfig,
};

const ISSUER: &str = "hash_route";

type JwtType = &'static str;

const REFRESH_TOKEN: JwtType = "refresh";
const ACCESS_TOKEN: JwtType = "access";

pub struct JWT {
    cfg: JWTConfig,
}

impl JWT {
    pub fn new(cfg: JWTConfig) -> Self {
        Self { cfg }
    }

    pub fn access_token(&self, id: sqlx::types::Uuid) -> Result<String, AuthError> {
        let id = match String::from_utf8(id.as_bytes().to_vec()) {
            Ok(id) => { id }
            Err(why) => {
                error!(why=?why,"id parsing failed");
                return Err(AuthError::Internal);
            }
        };

        let claims = Claims {
            exp: self.cfg.access_exp, // 1800 equivalent to 30mins-- one usize is equivalent to 1 sec
            token_type: ACCESS_TOKEN.to_owned(),
            iat: time::OffsetDateTime::now_utc().unix_timestamp() as usize,
            id: id.clone(),
            iss: ISSUER.to_owned(),
            sub: id,
        };

        let key = Keys::new(self.cfg.secrets.as_bytes());

        let token = match encode(&Header::default(), &claims, &key.encoding_key) {
            Ok(tk) => tk,
            Err(why) => {
                error!(why=?why,"failed to encode claim");
                return Err(AuthError::TokenCreation);
            }
        };

        Ok(token)
    }

    pub fn refresh_token(&self, id: sqlx::types::Uuid) -> Result<String, AuthError> {
        let id = match String::from_utf8(id.as_bytes().to_vec()) {
            Ok(id) => { id }
            Err(why) => {
                error!(why=?why,"id parsing failed");
                return Err(AuthError::InvalidID);
            }
        };

        let claims = Claims {
            exp: self.cfg.refresh_token_exp, // 7200 equivalent to 30mins-- one usize is equivalent to 1 sec
            token_type: REFRESH_TOKEN.to_owned(),
            iat: time::OffsetDateTime::now_utc().unix_timestamp() as usize,
            id: id.clone(),
            iss: ISSUER.to_owned(),
            sub: id,
        };

        let key = Keys::new(self.cfg.secrets.as_bytes());

        let token = match encode(&Header::default(), &claims, &key.encoding_key) {
            Ok(tk) => tk,
            Err(why) => {
                error!(why=?why,"failed to encode claim");
                return Err(AuthError::TokenCreation);
            }
        };

        Ok(token)
    }

    pub fn validate_access(&self, token_string: String) -> Result<Claims, AuthError> {
        let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::default());

        validation.set_issuer(&[ISSUER]);
        validation.set_required_spec_claims(&["exp", "iat", "sub", "iss", "nbf"]);

        let decoing_key = Keys::new(self.cfg.secrets.as_bytes().as_ref());

        let claims = match
            decode::<Claims>(token_string.as_bytes(), &decoing_key.decoding_key, &validation)
        {
            Ok(cl) => cl.claims,
            Err(why) => {
                error!(why=?why, "failed to decode user token");
                return Err(AuthError::InvalidToken);
            }
        };

        if claims.iss.trim().is_empty() {
            error!("claim has missing credential");
            return Err(AuthError::MissingCredentials);
        }
        if !claims.iss.trim().ends_with(ISSUER) {
            error!("claim has wrong credential");
            return Err(AuthError::WrongCredentails);
        }

        if !claims.token_type.ends_with(ACCESS_TOKEN) {
            error!("claim has wrong credential");
            return Err(AuthError::WrongCredentails);
        }

        if claims.exp > time::OffsetDateTime::now_utc().unix_timestamp() as usize {
            error!("token has expired");
            return Err(AuthError::WrongCredentails);
        }

        Ok(claims)
    }

    pub fn validate_refresh(&self, token_string: String) -> Result<Claims, AuthError> {
        let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::default());

        validation.set_issuer(&[ISSUER]);
        validation.set_required_spec_claims(&["exp", "iat", "sub", "iss", "nbf"]);

        let decoing_key = Keys::new(self.cfg.secrets.as_bytes().as_ref());

        let claims = match
            decode::<Claims>(token_string.as_bytes(), &decoing_key.decoding_key, &validation)
        {
            Ok(cl) => cl.claims,
            Err(why) => {
                error!(why=?why, "failed to decode user token");
                return Err(AuthError::InvalidToken);
            }
        };

        if claims.iss.trim().is_empty() {
            error!("claim has missing credential");
            return Err(AuthError::MissingCredentials);
        }
        if !claims.iss.trim().ends_with(ISSUER) {
            error!("claim has wrong credential");
            return Err(AuthError::WrongCredentails);
        }

        if !claims.token_type.ends_with(ACCESS_TOKEN) {
            error!("claim has wrong credential");
            return Err(AuthError::WrongCredentails);
        }

        if claims.exp > time::OffsetDateTime::now_utc().unix_timestamp() as usize {
            error!("token has expired");
            return Err(AuthError::WrongCredentails);
        }

        Ok(claims)
    }
}
