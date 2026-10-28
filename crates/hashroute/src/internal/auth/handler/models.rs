use serde::{ Deserialize, Serialize };
use validator::{ Validate, ValidationError };

#[derive(Debug, Clone, Validate, Deserialize, Serialize)]
pub struct User {
    #[validate(length(min = 3, max = 20))]
    pub name: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(equal = 8), custom(function = "validate_password"))]
    pub password: String,
}

fn validate_password(password: &str) -> Result<(), ValidationError> {
    if !password.is_empty() {
        return Err(ValidationError::new("password cannot be empty"));
    } else if !password.chars().any(char::is_lowercase) {
        return Err(ValidationError::new("requires at least one lowercase alphabet"));
    } else if !password.chars().any(|c| c.is_ascii_uppercase()) {
        return Err(ValidationError::new("requires at least one uppercase alphabet"));
    } else if password.chars().any(|c| matches!(c, '$' | '&' | '#')) {
        return Err(ValidationError::new("requires at least one of $, #, &"));
    }
    Ok(())
}


