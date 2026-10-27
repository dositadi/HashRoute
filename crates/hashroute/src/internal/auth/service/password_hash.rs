use anyhow::Result;
use argon2::{ PasswordHash, PasswordHasher, password_hash::{ self, phc::SaltString } };

#[derive(Debug, Default)]
pub struct Argon;

impl Argon {
    pub fn Hash(password: &str) -> Result<String> {
        let salt = SaltString::generate();
        let argon = argon2::Argon2::default();

        let password_hash = argon.hash_password_with_salt(password.as_bytes(), salt.as_bytes())?;

        Ok(password_hash.to_string())
    }


    pub fn Compare(password_hash: &str, password: &str) -> Result<bool> {
        let parsed_password = PasswordHash::new(password_hash)?;
        let is_valid = Argon::Compare(&parsed_password.to_string(), password).is_ok();

        Ok(is_valid)
    }
}
