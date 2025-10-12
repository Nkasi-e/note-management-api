#[cfg(feature = "azure")]
use async_trait::async_trait;
#[cfg(feature = "azure")]
use bytes::Bytes;
#[cfg(feature = "azure")]
use azure_storage::StorageCredentials;
#[cfg(feature = "azure")]
use azure_storage_blobs::prelude::*;
#[cfg(feature = "azure")]
use uuid::Uuid;
#[cfg(feature = "azure")]
use futures::StreamExt;
#[cfg(feature = "azure")]
use crate::domain::{Result, ApiError};
#[cfg(feature = "azure")]
use super::StorageProvider;

#[cfg(feature = "azure")]
/// Azure Blob Storage provider
#[derive(Clone)]
pub struct AzureStorage {
    container_client: ContainerClient,
    cdn_url: Option<String>,
}

#[cfg(feature = "azure")]
impl AzureStorage {
    pub fn new(
        account_name: String,
        account_key: String,
        container_name: String,
        cdn_url: Option<String>,
    ) -> Self {
        let credentials = StorageCredentials::access_key(account_name.clone(), account_key);
        let blob_service = BlobServiceClient::new(account_name, credentials);
        let container_client = blob_service.container_client(container_name);
        
        Self {
            container_client,
            cdn_url,
        }
    }
}

#[cfg(feature = "azure")]
#[async_trait]
impl StorageProvider for AzureStorage {
    async fn upload(
        &self,
        filename: &str,
        content_type: &str,
        data: Bytes,  // ← Zero-copy input
        user_id: Uuid,
    ) -> Result<String> {
        let blob_name = format!("users/{}/{}", user_id, filename);
        
        let blob_client = self.container_client.blob_client(&blob_name);
        
        // Azure SDK needs Vec, so convert (single copy)
        blob_client
            .put_block_blob(data.to_vec())
            .content_type(content_type)
            .execute()
            .await
            .map_err(|e| ApiError::InternalError(format!("Azure upload failed: {}", e)))?;
        
        tracing::info!("Uploaded file to Azure Blob Storage: {}", blob_name);
        
        Ok(blob_name)
    }

    async fn download(&self, key: &str) -> Result<Bytes> {
        let blob_client = self.container_client.blob_client(key);
        
        let mut stream = blob_client
            .get()
            .into_stream()
            .await
            .map_err(|e| ApiError::InternalError(format!("Azure download failed: {}", e)))?;
        
        let mut data = Vec::new();
        
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result
                .map_err(|e| ApiError::InternalError(format!("Failed to read chunk: {}", e)))?;
            data.extend_from_slice(&chunk.data);
        }
        
        // Convert to Bytes (cheap wrap)
        Ok(Bytes::from(data))
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let blob_client = self.container_client.blob_client(key);
        
        blob_client
            .delete()
            .execute()
            .await
            .map_err(|e| ApiError::InternalError(format!("Azure delete failed: {}", e)))?;
        
        tracing::info!("Deleted file from Azure Blob Storage: {}", key);
        
        Ok(())
    }

    fn get_url(&self, key: &str) -> String {
        if let Some(cdn_url) = &self.cdn_url {
            format!("{}/{}", cdn_url, key)
        } else {
            // Azure Blob Storage URL format
            let blob_client = self.container_client.blob_client(key);
            blob_client.url().to_string()
        }
    }

    async fn generate_presigned_url(
        &self,
        key: &str,
        expires_in: std::time::Duration,
    ) -> Result<String> {
        // Use the existing SAS URL generation method
        Self::generate_sas_url(self, key, expires_in).await
    }

    fn backend_name(&self) -> &str {
        "azure"
    }
}

#[cfg(feature = "azure")]
impl AzureStorage {
    /// Generate SAS (Shared Access Signature) URL for temporary access
    pub async fn generate_sas_url(
        &self,
        key: &str,
        expires_in: std::time::Duration,
    ) -> Result<String> {
        use azure_storage::shared_access_signature::SasProtocol;
        use chrono::{Duration, Utc};
        
        let expiry = Utc::now() + Duration::from_std(expires_in)
            .map_err(|e| ApiError::InternalError(format!("Invalid duration: {}", e)))?;
        
        let blob_client = self.container_client.blob_client(key);
        
        let sas_url = blob_client
            .generate_signed_blob_url(&BlobSasPermissions {
                read: true,
                write: false,
                delete: false,
                ..Default::default()
            })
            .protocol(SasProtocol::Https)
            .expires_on(expiry)
            .await
            .map_err(|e| ApiError::InternalError(format!("Failed to generate SAS URL: {}", e)))?;
        
        Ok(sas_url)
    }
}

