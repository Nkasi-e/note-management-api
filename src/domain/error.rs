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
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiError::UserNotFound { .. } => (StatusCode::NOT_FOUND, self.to_string()),
            ApiError::TaskNotFound { .. } => (StatusCode::NOT_FOUND, self.to_string()),
            ApiError::InvalidUuid(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            ApiError::EmailAlreadyExists { .. } => (StatusCode::CONFLICT, self.to_string()),
            ApiError::ValidationError(_) => (StatusCode::BAD_REQUEST, self.to_string()),
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
        ApiError::ValidationError(e.to_string())
    }
}
