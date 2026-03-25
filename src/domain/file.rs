use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;

/// File metadata stored in database
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct FileMetadata {
    pub id: Uuid,
    pub filename: String,
    pub original_filename: String,
    pub content_type: String,
    pub size: i64,
    pub path: String,
    pub uploaded_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request to upload a file
#[derive(Debug, Deserialize)]
pub struct UploadFileRequest {
    pub description: Option<String>,
}

/// Response after uploading a file
#[derive(Debug, Serialize, ToSchema)]
pub struct UploadFileResponse {
    pub id: Uuid,
    pub filename: String,
    pub original_filename: String,
    pub content_type: String,
    pub size: i64,
    pub url: String,
    pub created_at: DateTime<Utc>,
}

impl From<FileMetadata> for UploadFileResponse {
    fn from(metadata: FileMetadata) -> Self {
        Self {
            id: metadata.id,
            filename: metadata.filename.clone(),
            original_filename: metadata.original_filename,
            content_type: metadata.content_type,
            size: metadata.size,
            url: format!("/api/v1/files/{}", metadata.filename),
            created_at: metadata.created_at,
        }
    }
}

/// File upload statistics
#[derive(Debug, Serialize, ToSchema)]
pub struct FileStats {
    pub total_files: i64,
    pub total_size: i64,
    pub total_size_mb: f64,
}

