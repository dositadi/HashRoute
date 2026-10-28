use std::{ str::{ self } };

use anyhow::{ Result };
use axum::{ RequestPartsExt, extract::{ FromRef, FromRequestParts } };
use axum_extra::{ TypedHeader, headers::{ Authorization, authorization::Bearer } };
use jsonwebtoken::{ Header, decode, encode };
use tracing::error;

use crate::internal::{
    auth::service::models::{ AccessClaims, AuthError, JwtAccessTokenResponse, Keys, RefreshClaims },
    config::{ app_state::AppState, jwt_config::JWTConfig },
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

    pub fn access_token(&self, id: sqlx::types::Uuid) -> Result<JwtAccessTokenResponse, AuthError> {
        let id = match String::from_utf8(id.as_bytes().to_vec()) {
            Ok(id) => { id }
            Err(why) => {
                error!(why=?why,"id parsing failed");
                return Err(AuthError::Internal);
            }
        };

        let exp = self.cfg.access_exp;

        let claims = AccessClaims {
            exp, // 1800 equivalent to 30mins-- one usize is equivalent to 1 sec
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

        Ok(JwtAccessTokenResponse {
            access_token: token,
            expires_at: exp,
        })
    }

    pub fn refresh_token(&self, id: sqlx::types::Uuid) -> Result<String, AuthError> {
        let id = match String::from_utf8(id.as_bytes().to_vec()) {
            Ok(id) => { id }
            Err(why) => {
                error!(why=?why,"id parsing failed");
                return Err(AuthError::InvalidID);
            }
        };

        let claims = RefreshClaims {
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

    pub fn validate_access(&self, token_string: String) -> Result<AccessClaims, AuthError> {
        let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::default());

        validation.set_issuer(&[ISSUER]);
        validation.set_required_spec_claims(&["exp", "iat", "sub", "iss", "nbf"]);

        let decoding_key = Keys::new(self.cfg.secrets.as_bytes());

        let claims = match
            decode::<AccessClaims>(token_string.as_bytes(), &decoding_key.decoding_key, &validation)
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

        if claims.exp > (time::OffsetDateTime::now_utc().unix_timestamp() as usize) {
            error!("token has expired");
            return Err(AuthError::WrongCredentails);
        }

        Ok(claims)
    }

    pub fn validate_refresh(&self, token_string: String) -> Result<RefreshClaims, AuthError> {
        let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::default());

        validation.set_issuer(&[ISSUER]);
        validation.set_required_spec_claims(&["exp", "iat", "sub", "iss", "nbf"]);

        let decoing_key = Keys::new(self.cfg.secrets.as_bytes());

        let claims = match
            decode::<RefreshClaims>(token_string.as_bytes(), &decoing_key.decoding_key, &validation)
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

        if claims.exp > (time::OffsetDateTime::now_utc().unix_timestamp() as usize) {
            error!("token has expired");
            return Err(AuthError::WrongCredentails);
        }

        Ok(claims)
    }
}

impl<S> FromRequestParts<S>
    for AccessClaims
    where
        AppState: FromRef<S>, // Tells axum to extract the appstate from the state
        S: Send + Sync
{
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S
    ) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>().await
            .map_err(|_| AuthError::InvalidToken)?;

        let app_state = AppState::from_ref(state); //Converts the stateto appstate type

        let jwt = JWT::new(app_state.jwt_secret);

        let claims = match jwt.validate_access(bearer.token().to_owned()) {
            Err(why) => {
                return Err(why);
            }
            Ok(claims) => claims,
        };

        Ok(claims)
    }
}

impl<S> FromRequestParts<S> for RefreshClaims where S: Send + Sync, AppState: FromRef<S> {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S
    ) -> std::prelude::v1::Result<Self, Self::Rejection> {
        let Some(value) = parts.headers.get("X-REFRESH") else {
            return Err(AuthError::InvalidToken);
        };

        let Ok(token) = value.to_str() else {
            return Err(AuthError::InvalidToken);
        };

        let app_state = AppState::from_ref(state);

        let jwt = JWT::new(app_state.jwt_secret);

        let refresh_claims = jwt.validate_refresh(token.to_owned())?;

        Ok(refresh_claims)
    }
}
