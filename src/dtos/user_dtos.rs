use std::borrow::Cow;

use chrono::NaiveDateTime;
use regex::Regex;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidateEmail, ValidationError};

use crate::models::user_model::User;

#[derive(Validate, Debug, Default, Clone, Serialize, Deserialize)]
pub struct RegisterUserDto {
    #[validate(length(min = 3, message = "Username must be at least 3 characters long"))]
    #[validate(custom(function = "validate_username"))]
    pub username: String,
    #[validate(
        length(min = 1, message = "Email is required"),
        email(message = "Email is invalid")
    )]
    pub email: String,
    #[validate(length(min = 6, message = "Password must be at least 6 characters"))]
    pub password: String,

    #[validate(
        length(min = 1, message = "Confirm Password is required"),
        must_match(other = "password", message = "passwords do not match")
    )]
    #[serde(rename = "passwordConfirm")]
    pub password_confirm: String,
}

fn validate_username(username: &str) -> Result<(), ValidationError> {
    let re = Regex::new(r"^[a-zA-Z0-9_]+$").unwrap();
    if !re.is_match(username) {
        return Err(ValidationError::new(
            "Username can only contain letters, numbers, and underscores",
        ));
    }

    Ok(())
}

#[derive(Validate, Debug, Default, Clone, Serialize, Deserialize)]
pub struct LoginUserDto {
    #[validate(custom(function = "validate_identifier"))]
    pub identifier: String,
    #[validate(length(min = 6, message = "Password must be at least 6 characters"))]
    pub password: String,
}

struct EmailValidator {
    identifier: String,
}

impl EmailValidator {
    fn is_email(&self) -> bool {
        self.identifier.contains('@')
    }

    fn is_wrong_length(&self) -> bool {
        self.identifier.len() < 3
    }
}

impl ValidateEmail for EmailValidator {
    fn as_email_string(&self) -> Option<Cow<'_, str>> {
        if self.identifier.contains('@') {
            Some(self.identifier.clone().into())
        } else {
            None
        }
    }
}

fn validate_identifier(identifier: &str) -> Result<(), ValidationError> {
    // You can define your own logic to differentiate between email and username
    let email_validator = EmailValidator {
        identifier: identifier.to_string(),
    };
    if email_validator.is_email() {
        if !email_validator.validate_email() {
            return Err(ValidationError::new("Invalid email format"));
        }
    } else if email_validator.is_wrong_length() {
        return Err(ValidationError::new(
            "Username must be at least 3 characters long",
        ));
    }
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FilterUserDto {
    pub id: String,
    pub username: String,
    pub email: String,
    #[serde(rename = "createdAt")]
    pub created_at: NaiveDateTime,
    #[serde(rename = "updatedAt")]
    pub updated_at: NaiveDateTime,
}

impl FilterUserDto {
    pub fn filter_user(user: &User) -> Self {
        FilterUserDto {
            id: user.id.to_string(),
            username: user.username.to_owned(),
            email: user.email.to_owned(),
            created_at: user.created_at.unwrap(),
            updated_at: user.updated_at.unwrap(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserLoginResponseDto {
    pub status: String,
    pub user: FilterUserDto,
    pub token: String,
    pub refresh_token: String,
}

#[derive(Serialize, Deserialize)]
pub struct Response {
    pub status: &'static str,
    pub message: String,
}
