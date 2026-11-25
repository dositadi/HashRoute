use std::borrow::Cow;

use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

#[derive(Debug, Clone, Validate, Deserialize, Serialize)]
pub struct RegisterPayload {
    #[validate(length(min = 3, max = 20, message = "name must be 3 to 20 characters"))]
    pub name: String,
    #[validate(email(message = "email is not valid"))]
    pub email: String,
    #[validate(
        length(min = 8, message = "password must be at least 8 characters"),
        custom(function = "validate_password")
    )]
    pub password: String,
}

fn rule(message: &'static str) -> ValidationError {
    ValidationError::new("password_rule").with_message(Cow::Borrowed(message))
}

fn validate_password(password: &str) -> Result<(), ValidationError> {
    if !password.chars().any(char::is_lowercase) {
        return Err(rule("password needs at least one lowercase letter"));
    }
    if !password.chars().any(|c| c.is_ascii_uppercase()) {
        return Err(rule("password needs at least one uppercase letter"));
    }
    if !password.chars().any(|c| matches!(c, '$' | '&' | '#')) {
        return Err(rule("password needs at least one of $, # or &"));
    }
    Ok(())
}

/// Login only checks that fields are present. Strength rules belong to registration; a wrong
/// password is reported as invalid credentials, not as a validation error.
#[derive(Debug, Clone, Validate, Deserialize)]
pub struct LoginPayload {
    #[validate(email(message = "email is not valid"))]
    pub email: String,
    #[validate(length(min = 1, message = "password is required"))]
    pub password: String,
}
