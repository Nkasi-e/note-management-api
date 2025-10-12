use async_trait::async_trait;
use bytes::Bytes;
use uuid::Uuid;
use crate::domain::{Result, ApiError};
use super::StorageProvider;

/// Local filesystem storage provider
#[derive(Clone)]
pub struct LocalStorage {
    upload_dir: String,
}

impl LocalStorage {
    pub fn new(upload_dir: String) -> Self {
        Self { upload_dir }
    }

    /// Ensure upload directory exists
    pub async fn init(&self) -> Result<()> {
        tokio::fs::create_dir_all(&self.upload_dir)
            .await
            .map_err(|e| ApiError::InternalError(format!("Failed to create upload directory: {}", e)))?;
        Ok(())
    }
}

#[async_trait]
impl StorageProvider for LocalStorage {
    async fn upload(
        &self,
        filename: &str,
        _content_type: &str,
        data: Bytes,  // ← Zero-copy input
        user_id: Uuid,
    ) -> Result<String> {
        // Create user directory
        let user_dir = format!("{}/{}", self.upload_dir, user_id);
        tokio::fs::create_dir_all(&user_dir)
            .await
            .map_err(|e| ApiError::InternalError(format!("Failed to create user directory: {}", e)))?;

        // Full path
        let path = format!("{}/{}", user_dir, filename);
        
        // Write file directly from Bytes (no copy needed!)
        tokio::fs::write(&path, &data)
            .await
            .map_err(|e| ApiError::InternalError(format!("Failed to write file: {}", e)))?;

        // Return relative path
        Ok(format!("{}/{}", user_id, filename))
    }

    async fn download(&self, key: &str) -> Result<Bytes> {
        let path = format!("{}/{}", self.upload_dir, key);
        
        // Read into Vec<u8> then convert to Bytes (single allocation)
        let data = tokio::fs::read(&path)
            .await
            .map_err(|e| ApiError::InternalError(format!("Failed to read file: {}", e)))?;
        
        // Convert to Bytes - this is cheap, just wraps the Vec
        Ok(Bytes::from(data))
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let path = format!("{}/{}", self.upload_dir, key);
        
        tokio::fs::remove_file(&path)
            .await
            .map_err(|e| ApiError::InternalError(format!("Failed to delete file: {}", e)))
    }

    fn get_url(&self, key: &str) -> String {
        // For local storage, return API endpoint URL
        // The file is served through the regular download endpoint
        // Key format is: user_id/filename, we need file_id instead
        // This is a simplified implementation - in production you'd look up file_id
        format!("/api/v1/files/{}", key)
    }

    async fn generate_presigned_url(
        &self,
        key: &str,
        _expires_in: std::time::Duration,
    ) -> Result<String> {
        // Local storage doesn't support presigned URLs
        // Return the regular URL instead
        // Note: In production, you could implement JWT-based temporary tokens
        tracing::warn!(
            "Local storage doesn't support presigned URLs, returning regular URL for: {}",
            key
        );
        Ok(self.get_url(key))
    }

    fn backend_name(&self) -> &str {
        "local"
    }
}

