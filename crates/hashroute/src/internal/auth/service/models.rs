use jsonwebtoken::{ DecodingKey, EncodingKey };
use serde::{ Deserialize, Serialize };

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AccessClaims {
    pub id: String,
    pub token_type: String,
    pub sub: String,
    pub exp: usize,
    pub iss: String,
    pub iat: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RefreshClaims {
    pub id: String,
    pub token_type: String,
    pub sub: String,
    pub exp: usize,
    pub iss: String,
    pub iat: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JwtAccessTokenResponse {
    pub access_token: String,
    pub expires_at: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthPayload {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug)]
pub struct Keys {
    pub encoding_key: EncodingKey,
    pub decoding_key: DecodingKey,
}

impl Keys {
    pub fn new(secret: &[u8]) -> Keys {
        Keys {
            encoding_key: EncodingKey::from_secret(secret),
            decoding_key: DecodingKey::from_secret(secret),
        }
    }
}
