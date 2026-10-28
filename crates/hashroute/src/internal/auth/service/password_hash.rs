use anyhow::{ Result };
use argon2::{
    Argon2,
    PasswordHash,
    PasswordHasher,
    PasswordVerifier,
    password_hash::{ phc::SaltString },
};

use crate::internal::auth::service::models::AuthError;

#[derive(Debug, Default)]
pub struct Argon;

impl Argon {
    pub fn hash(&self, password: &str) -> Result<String> {
        let salt = SaltString::generate();
        let argon = argon2::Argon2::default();

        let password_hash = argon.hash_password_with_salt(password.as_bytes(), salt.as_bytes())?;

        Ok(password_hash.to_string())
    }

    pub fn compare(&self, password_hash: &str, password: &str) -> Result<bool, AuthError> {
        let Ok(parsed_password) = PasswordHash::new(password_hash) else {return  Err(AuthError::Internal);};

        if Argon2::default().verify_password(password.as_bytes(), &parsed_password).is_err() {
            return Err(AuthError::InvalidPassword)
        };
        

        Ok(true)
    }
}
