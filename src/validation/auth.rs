use regex::Regex;
use crate::domain::{ApiError, Result};
use crate::services::auth_service::{RegisterRequest, LoginRequest};

pub struct Validator;

impl Validator {
    pub fn validate_register_request(req: &RegisterRequest) -> Result<()> {
        // Validate name
        if req.name.trim().is_empty() {
            return Err(ApiError::validation_error("Name is required and cannot be empty"));
        }
        if req.name.trim().len() < 2 {
            return Err(ApiError::validation_error("Name must be at least 2 characters long"));
        }
        if req.name.trim().len() > 100 {
            return Err(ApiError::validation_error("Name cannot exceed 100 characters"));
        }

        // Validate email
        if req.email.trim().is_empty() {
            return Err(ApiError::validation_error("Email is required and cannot be empty"));
        }
        if !Self::is_valid_email(&req.email) {
            return Err(ApiError::validation_error("Please provide a valid email address"));
        }
        if req.email.len() > 255 {
            return Err(ApiError::validation_error("Email cannot exceed 255 characters"));
        }

        // Validate password
        if req.password.is_empty() {
            return Err(ApiError::validation_error("Password is required and cannot be empty"));
        }
        if req.password.len() < 8 {
            return Err(ApiError::validation_error("Password must be at least 8 characters long"));
        }
        if req.password.len() > 128 {
            return Err(ApiError::validation_error("Password cannot exceed 128 characters"));
        }
        if !Self::is_strong_password(&req.password) {
            return Err(ApiError::validation_error("Password must contain at least one uppercase letter, one lowercase letter, and one number"));
        }

        Ok(())
    }

    pub fn validate_login_request(req: &LoginRequest) -> Result<()> {
        // Validate email
        if req.email.trim().is_empty() {
            return Err(ApiError::validation_error("Email is required and cannot be empty"));
        }
        if !Self::is_valid_email(&req.email) {
            return Err(ApiError::validation_error("Please provide a valid email address"));
        }

        // Validate password
        if req.password.is_empty() {
            return Err(ApiError::validation_error("Password is required and cannot be empty"));
        }

        Ok(())
    }

    fn is_valid_email(email: &str) -> bool {
        let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
        email_regex.is_match(email.trim())
    }

    fn is_strong_password(password: &str) -> bool {
        let has_uppercase = password.chars().any(|c| c.is_uppercase());
        let has_lowercase = password.chars().any(|c| c.is_lowercase());
        let has_digit = password.chars().any(|c| c.is_ascii_digit());
        
        has_uppercase && has_lowercase && has_digit
    }
}
