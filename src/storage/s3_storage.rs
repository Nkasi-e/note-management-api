#[cfg(feature = "s3")]
use async_trait::async_trait;
#[cfg(feature = "s3")]
use aws_sdk_s3::{Client, primitives::ByteStream};
#[cfg(feature = "s3")]
use aws_config::meta::region::RegionProviderChain;
#[cfg(feature = "s3")]
use uuid::Uuid;
#[cfg(feature = "s3")]
use crate::domain::{Result, ApiError};
#[cfg(feature = "s3")]
use super::StorageProvider;

#[cfg(feature = "s3")]

/// AWS S3 storage provider
#[derive(Clone)]
pub struct S3Storage {
    client: Client,
    bucket: String,
    cdn_url: Option<String>,
}

impl S3Storage {
    pub async fn new(bucket: String, region: String, cdn_url: Option<String>) -> Self {
        let region_provider = RegionProviderChain::default_provider()
            .or_else(region.as_str());
        
        let config = aws_config::from_env()
            .region(region_provider)
            .load()
            .await;
        
        let client = Client::new(&config);
        
        Self {
            client,
            bucket,
            cdn_url,
        }
    }

    /// Generate presigned URL for direct client upload (optional advanced feature)
    pub async fn generate_presigned_url(
        &self,
        key: &str,
        expires_in: std::time::Duration,
    ) -> Result<String> {
        let presigning_config = aws_sdk_s3::presigning::PresigningConfig::expires_in(expires_in)
            .map_err(|e| ApiError::InternalError(format!("Presigning config error: {}", e)))?;
        
        let presigned_request = self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .presigned(presigning_config)
            .await
            .map_err(|e| ApiError::InternalError(format!("Presigning failed: {}", e)))?;
        
        Ok(presigned_request.uri().to_string())
    }
}

#[async_trait]
impl StorageProvider for S3Storage {
    async fn upload(
        &self,
        filename: &str,
        content_type: &str,
        data: Vec<u8>,
        user_id: Uuid,
    ) -> Result<String> {
        // Generate S3 key with user folder structure
        let key = format!("users/{}/{}", user_id, filename);
        
        // Upload to S3
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .body(ByteStream::from(data))
            .content_type(content_type)
            .send()
            .await
            .map_err(|e| ApiError::InternalError(format!("S3 upload failed: {}", e)))?;
        
        tracing::info!("Uploaded file to S3: s3://{}/{}", self.bucket, key);
        
        Ok(key)
    }

    async fn download(&self, key: &str) -> Result<Vec<u8>> {
        let response = self.client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| ApiError::InternalError(format!("S3 download failed: {}", e)))?;
        
        let data = response.body.collect().await
            .map_err(|e| ApiError::InternalError(format!("Failed to read S3 object: {}", e)))?;
        
        Ok(data.into_bytes().to_vec())
    }

    async fn delete(&self, key: &str) -> Result<()> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| ApiError::InternalError(format!("S3 delete failed: {}", e)))?;
        
        tracing::info!("Deleted file from S3: s3://{}/{}", self.bucket, key);
        
        Ok(())
    }

    fn get_url(&self, key: &str) -> String {
        if let Some(cdn_url) = &self.cdn_url {
            // Use CloudFront URL if configured
            format!("{}/{}", cdn_url, key)
        } else {
            // Use S3 direct URL
            format!("https://{}.s3.amazonaws.com/{}", self.bucket, key)
        }
    }

    async fn generate_presigned_url(
        &self,
        key: &str,
        expires_in: std::time::Duration,
    ) -> Result<String> {
        // Use the existing method
        Self::generate_presigned_url(self, key, expires_in).await
    }

    fn backend_name(&self) -> &str {
        "s3"
    }
}

