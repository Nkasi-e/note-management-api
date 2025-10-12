#[cfg(feature = "gcs")]
use async_trait::async_trait;
#[cfg(feature = "gcs")]
use google_cloud_storage::client::{Client, ClientConfig};
#[cfg(feature = "gcs")]
use google_cloud_storage::http::objects::upload::{UploadObjectRequest, UploadType, Media};
#[cfg(feature = "gcs")]
use google_cloud_storage::http::objects::get::GetObjectRequest;
#[cfg(feature = "gcs")]
use google_cloud_storage::http::objects::delete::DeleteObjectRequest;
#[cfg(feature = "gcs")]
use uuid::Uuid;
#[cfg(feature = "gcs")]
use crate::domain::{Result, ApiError};
#[cfg(feature = "gcs")]
use super::StorageProvider;

#[cfg(feature = "gcs")]
/// Google Cloud Storage provider
#[derive(Clone)]
pub struct GcsStorage {
    client: Client,
    bucket: String,
    cdn_url: Option<String>,
}

#[cfg(feature = "gcs")]
impl GcsStorage {
    pub async fn new(bucket: String, cdn_url: Option<String>) -> Result<Self> {
        let config = ClientConfig::default()
            .with_auth()
            .await
            .map_err(|e| ApiError::InternalError(format!("GCS auth failed: {}", e)))?;
        
        let client = Client::new(config);
        
        Ok(Self {
            client,
            bucket,
            cdn_url,
        })
    }
}

#[cfg(feature = "gcs")]
#[async_trait]
impl StorageProvider for GcsStorage {
    async fn upload(
        &self,
        filename: &str,
        content_type: &str,
        data: Vec<u8>,
        user_id: Uuid,
    ) -> Result<String> {
        let key = format!("users/{}/{}", user_id, filename);
        
        let upload_type = UploadType::Simple(Media {
            name: key.clone().into(),
            content_type: content_type.to_string().into(),
            content_length: Some(data.len()),
        });
        
        let request = UploadObjectRequest {
            bucket: self.bucket.clone(),
            ..Default::default()
        };
        
        self.client
            .upload_object(&request, data, &upload_type)
            .await
            .map_err(|e| ApiError::InternalError(format!("GCS upload failed: {}", e)))?;
        
        tracing::info!("Uploaded file to GCS: gs://{}/{}", self.bucket, key);
        
        Ok(key)
    }

    async fn download(&self, key: &str) -> Result<Vec<u8>> {
        let request = GetObjectRequest {
            bucket: self.bucket.clone(),
            object: key.to_string(),
            ..Default::default()
        };
        
        let data = self.client
            .download_object(&request, &Default::default())
            .await
            .map_err(|e| ApiError::InternalError(format!("GCS download failed: {}", e)))?;
        
        Ok(data)
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let request = DeleteObjectRequest {
            bucket: self.bucket.clone(),
            object: key.to_string(),
            ..Default::default()
        };
        
        self.client
            .delete_object(&request)
            .await
            .map_err(|e| ApiError::InternalError(format!("GCS delete failed: {}", e)))?;
        
        tracing::info!("Deleted file from GCS: gs://{}/{}", self.bucket, key);
        
        Ok(())
    }

    fn get_url(&self, key: &str) -> String {
        if let Some(cdn_url) = &self.cdn_url {
            format!("{}/{}", cdn_url, key)
        } else {
            format!("https://storage.googleapis.com/{}/{}", self.bucket, key)
        }
    }

    async fn generate_presigned_url(
        &self,
        key: &str,
        expires_in: std::time::Duration,
    ) -> Result<String> {
        use google_cloud_storage::http::objects::download::DownloadObjectRequest;
        use google_cloud_storage::sign::{SignedURLOptions, SignedURLMethod};
        
        let expiration = std::time::SystemTime::now() + expires_in;
        
        let options = SignedURLOptions {
            method: SignedURLMethod::GET,
            expires: expiration,
            ..Default::default()
        };
        
        let signed_url = self.client
            .signed_url(&self.bucket, key, None, None, options)
            .await
            .map_err(|e| ApiError::InternalError(format!("GCS presigned URL failed: {}", e)))?;
        
        tracing::info!("Generated GCS presigned URL for: gs://{}/{}", self.bucket, key);
        
        Ok(signed_url)
    }

    fn backend_name(&self) -> &str {
        "gcs"
    }
}

