use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("User not found: {id}")]
    UserNotFound { id: Uuid },
    
    #[error("Task not found: {id}")]
    TaskNotFound { id: Uuid },
    
    #[error("Invalid UUID: {0}")]
    InvalidUuid(String),
    
    #[error("Email already exists: {email}")]
    EmailAlreadyExists { email: String },
    
    #[error("Internal server error: {0}")]
    InternalError(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Forbidden: {0}")]
    Forbidden(String),
    
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    
    #[error("Bad request: {0}")]
    BadRequest(String),
    
    #[error("Conflict: {0}")]
    Conflict(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiError::UserNotFound { .. } => (StatusCode::NOT_FOUND, self.to_string()),
            ApiError::TaskNotFound { .. } => (StatusCode::NOT_FOUND, self.to_string()),
            ApiError::NotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            ApiError::InvalidUuid(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            ApiError::BadRequest(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            ApiError::EmailAlreadyExists { .. } => (StatusCode::CONFLICT, self.to_string()),
            ApiError::Conflict(_) => (StatusCode::CONFLICT, self.to_string()),
            ApiError::ValidationError(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            ApiError::Forbidden(_) => (StatusCode::FORBIDDEN, self.to_string()),
            ApiError::Unauthorized(_) => (StatusCode::UNAUTHORIZED, self.to_string()),
            ApiError::InternalError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = Json(json!({
            "success": false,
            "error": error_message,
            "status": status.as_u16()
        }));

        (status, body).into_response()
    }
}

pub type Result<T> = std::result::Result<T, ApiError>;

// Mappers
impl From<sqlx::Error> for ApiError {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::RowNotFound => ApiError::InternalError("Resource not found".into()),
            _ => ApiError::InternalError(format!("Database error: {}", e)),
        }
    }
}

impl From<axum::extract::rejection::JsonRejection> for ApiError {
    fn from(e: axum::extract::rejection::JsonRejection) -> Self {
        match e {
            axum::extract::rejection::JsonRejection::JsonDataError(err) => {
                ApiError::validation_error(format!("Invalid JSON data: {}", err))
            }
            axum::extract::rejection::JsonRejection::JsonSyntaxError(err) => {
                ApiError::validation_error(format!("Invalid JSON syntax: {}", err))
            }
            axum::extract::rejection::JsonRejection::MissingJsonContentType(_) => {
                ApiError::validation_error("Missing Content-Type: application/json header")
            }
            _ => ApiError::validation_error(format!("JSON parsing error: {}", e)),
        }
    }
}

// Helper functions for common error cases
impl ApiError {
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::Unauthorized(message.into())
    }
    
    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::Forbidden(message.into())
    }
    
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::BadRequest(message.into())
    }
    
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::NotFound(message.into())
    }
    
    pub fn conflict(message: impl Into<String>) -> Self {
        Self::Conflict(message.into())
    }
    
    pub fn validation_error(message: impl Into<String>) -> Self {
        Self::ValidationError(message.into())
    }
    
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::InternalError(message.into())
    }
}
