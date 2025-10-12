pub mod local_storage;

#[cfg(feature = "s3")]
pub mod s3_storage;

#[cfg(feature = "gcs")]
pub mod gcs_storage;

#[cfg(feature = "cloudinary")]
pub mod cloudinary_storage;

#[cfg(feature = "azure")]
pub mod azure_storage;

use async_trait::async_trait;
use bytes::Bytes;
use uuid::Uuid;
use crate::domain::{Result, ApiError};
use local_storage::LocalStorage;

#[cfg(feature = "s3")]
use s3_storage::S3Storage;

#[cfg(feature = "gcs")]
use gcs_storage::GcsStorage;

#[cfg(feature = "cloudinary")]
use cloudinary_storage::CloudinaryStorage;

#[cfg(feature = "azure")]
use azure_storage::AzureStorage;

/// Storage provider trait - abstraction for all storage backends
/// Uses `Bytes` for zero-copy performance
#[async_trait]
pub trait StorageProvider: Send + Sync {
    /// Upload a file and return the storage key/path
    /// Uses Bytes for zero-copy - just increments reference count instead of copying data
    async fn upload(
        &self,
        filename: &str,
        content_type: &str,
        data: Bytes,  // ← Zero-copy: reference-counted buffer
        user_id: Uuid,
    ) -> Result<String>;

    /// Download a file by its key/path
    /// Returns Bytes for zero-copy streaming
    async fn download(&self, key: &str) -> Result<Bytes>;  // ← Zero-copy output

    /// Delete a file by its key/path
    async fn delete(&self, key: &str) -> Result<()>;

    /// Get the public URL for a file
    fn get_url(&self, key: &str) -> String;

    /// Generate a presigned URL for temporary access
    /// Returns a URL that grants time-limited access without authentication
    async fn generate_presigned_url(
        &self,
        key: &str,
        expires_in: std::time::Duration,
    ) -> Result<String>;

    /// Get the storage backend name
    fn backend_name(&self) -> &str;
}

/// Storage factory - creates the appropriate storage provider based on config
pub struct StorageFactory;

impl StorageFactory {
    pub async fn create(config: &crate::config::settings::StorageConfig) -> Result<Box<dyn StorageProvider>> {
        match config.backend.as_str() {
            "local" => {
                Ok(Box::new(LocalStorage::new(config.upload_dir.clone())))
            }
            #[cfg(feature = "s3")]
            "s3" => {
                let region = config.aws_region.clone()
                    .ok_or_else(|| ApiError::InternalError("AWS_REGION required for S3 backend".to_string()))?;
                let bucket = config.aws_bucket.clone()
                    .ok_or_else(|| ApiError::InternalError("AWS_S3_BUCKET required for S3 backend".to_string()))?;
                let cdn_url = config.aws_cdn_url.clone();
                
                Ok(Box::new(S3Storage::new(bucket, region, cdn_url).await))
            }
            #[cfg(not(feature = "s3"))]
            "s3" => {
                Err(ApiError::InternalError(
                    "S3 backend not available. Compile with --features s3".to_string()
                ))
            }
            
            #[cfg(feature = "gcs")]
            "gcs" => {
                let bucket = config.gcs_bucket.clone()
                    .ok_or_else(|| ApiError::InternalError("GCP_BUCKET required for GCS backend".to_string()))?;
                let cdn_url = config.gcs_cdn_url.clone();
                
                Ok(Box::new(GcsStorage::new(bucket, cdn_url).await?))
            }
            #[cfg(not(feature = "gcs"))]
            "gcs" => {
                Err(ApiError::InternalError(
                    "GCS backend not available. Compile with --features gcs".to_string()
                ))
            }
            
            #[cfg(feature = "cloudinary")]
            "cloudinary" => {
                let cloud_name = config.cloudinary_cloud_name.clone()
                    .ok_or_else(|| ApiError::InternalError("CLOUDINARY_CLOUD_NAME required".to_string()))?;
                let api_key = config.cloudinary_api_key.clone()
                    .ok_or_else(|| ApiError::InternalError("CLOUDINARY_API_KEY required".to_string()))?;
                let api_secret = config.cloudinary_api_secret.clone()
                    .ok_or_else(|| ApiError::InternalError("CLOUDINARY_API_SECRET required".to_string()))?;
                
                Ok(Box::new(CloudinaryStorage::new(cloud_name, api_key, api_secret)))
            }
            #[cfg(not(feature = "cloudinary"))]
            "cloudinary" => {
                Err(ApiError::InternalError(
                    "Cloudinary backend not available. Compile with --features cloudinary".to_string()
                ))
            }
            
            #[cfg(feature = "azure")]
            "azure" => {
                let account_name = config.azure_account_name.clone()
                    .ok_or_else(|| ApiError::InternalError("AZURE_ACCOUNT_NAME required".to_string()))?;
                let account_key = config.azure_account_key.clone()
                    .ok_or_else(|| ApiError::InternalError("AZURE_ACCOUNT_KEY required".to_string()))?;
                let container = config.azure_container.clone()
                    .ok_or_else(|| ApiError::InternalError("AZURE_CONTAINER required".to_string()))?;
                let cdn_url = config.azure_cdn_url.clone();
                
                Ok(Box::new(AzureStorage::new(account_name, account_key, container, cdn_url)))
            }
            #[cfg(not(feature = "azure"))]
            "azure" => {
                Err(ApiError::InternalError(
                    "Azure backend not available. Compile with --features azure".to_string()
                ))
            }
            
            backend => {
                Err(ApiError::InternalError(format!("Unsupported storage backend: {}", backend)))
            }
        }
    }
}

