use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{ApiError, Result, FileMetadata, FileStats};

#[derive(Debug, Clone)]
pub struct FileRepository {
    pool: PgPool,
}

impl FileRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Save file metadata to database
    pub async fn create(&self, metadata: FileMetadata) -> Result<FileMetadata> {
        let file = sqlx::query_as::<_, FileMetadata>(
            r#"
            INSERT INTO files (id, filename, original_filename, content_type, size, path, uploaded_by, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#
        )
        .bind(metadata.id)
        .bind(&metadata.filename)
        .bind(&metadata.original_filename)
        .bind(&metadata.content_type)
        .bind(metadata.size)
        .bind(&metadata.path)
        .bind(metadata.uploaded_by)
        .bind(metadata.created_at)
        .bind(metadata.updated_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ApiError::InternalError(format!("Database error: {}", e)))?;

        Ok(file)
    }

    /// Find file by ID
    pub async fn find_by_id(&self, id: Uuid) -> Result<FileMetadata> {
        let file = sqlx::query_as::<_, FileMetadata>(
            "SELECT * FROM files WHERE id = $1"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => ApiError::NotFound(format!("File with id {} not found", id)),
            _ => ApiError::InternalError(format!("Database error: {}", e)),
        })?;

        Ok(file)
    }

    /// Find file by filename
    pub async fn find_by_filename(&self, filename: &str) -> Result<FileMetadata> {
        let file = sqlx::query_as::<_, FileMetadata>(
            "SELECT * FROM files WHERE filename = $1"
        )
        .bind(filename)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => ApiError::NotFound(format!("File {} not found", filename)),
            _ => ApiError::InternalError(format!("Database error: {}", e)),
        })?;

        Ok(file)
    }

    /// List all files uploaded by a user
    pub async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<FileMetadata>> {
        let files = sqlx::query_as::<_, FileMetadata>(
            "SELECT * FROM files WHERE uploaded_by = $1 ORDER BY created_at DESC"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ApiError::InternalError(format!("Database error: {}", e)))?;

        Ok(files)
    }

    /// Delete file metadata
    pub async fn delete(&self, id: Uuid) -> Result<()> {
        let result = sqlx::query("DELETE FROM files WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| ApiError::InternalError(format!("Database error: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(ApiError::NotFound(format!("File with id {} not found", id)));
        }

        Ok(())
    }

    /// Get file statistics for a user
    pub async fn get_user_stats(&self, user_id: Uuid) -> Result<FileStats> {
        let stats = sqlx::query_as::<_, (i64, Option<i64>)>(
            "SELECT COUNT(*), COALESCE(SUM(size), 0) FROM files WHERE uploaded_by = $1"
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ApiError::InternalError(format!("Database error: {}", e)))?;

        let total_size = stats.1.unwrap_or(0);
        Ok(FileStats {
            total_files: stats.0,
            total_size,
            total_size_mb: total_size as f64 / (1024.0 * 1024.0),
        })
    }

    /// Check if user owns the file
    pub async fn is_owner(&self, file_id: Uuid, user_id: Uuid) -> Result<bool> {
        let result = sqlx::query_as::<_, (bool,)>(
            "SELECT EXISTS(SELECT 1 FROM files WHERE id = $1 AND uploaded_by = $2)"
        )
        .bind(file_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ApiError::InternalError(format!("Database error: {}", e)))?;

        Ok(result.0)
    }
}

