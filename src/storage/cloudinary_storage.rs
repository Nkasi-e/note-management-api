#[cfg(feature = "cloudinary")]
use async_trait::async_trait;
#[cfg(feature = "cloudinary")]
use reqwest::multipart::{Form, Part};
#[cfg(feature = "cloudinary")]
use uuid::Uuid;
#[cfg(feature = "cloudinary")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "cloudinary")]
use crate::domain::{Result, ApiError};
#[cfg(feature = "cloudinary")]
use super::StorageProvider;

#[cfg(feature = "cloudinary")]
#[derive(Debug, Deserialize)]
struct CloudinaryUploadResponse {
    public_id: String,
    secure_url: String,
    url: String,
    format: String,
    resource_type: String,
}

#[cfg(feature = "cloudinary")]
#[derive(Debug, Deserialize)]
struct CloudinaryDeleteResponse {
    result: String,
}

#[cfg(feature = "cloudinary")]
/// Cloudinary storage provider (best for images and videos)
#[derive(Clone)]
pub struct CloudinaryStorage {
    cloud_name: String,
    api_key: String,
    api_secret: String,
    client: reqwest::Client,
}

#[cfg(feature = "cloudinary")]
impl CloudinaryStorage {
    pub fn new(cloud_name: String, api_key: String, api_secret: String) -> Self {
        Self {
            cloud_name,
            api_key,
            api_secret,
            client: reqwest::Client::new(),
        }
    }

    fn get_upload_url(&self) -> String {
        format!("https://api.cloudinary.com/v1_1/{}/auto/upload", self.cloud_name)
    }

    fn get_delete_url(&self, resource_type: &str) -> String {
        format!(
            "https://api.cloudinary.com/v1_1/{}/{}/destroy",
            self.cloud_name, resource_type
        )
    }

    /// Generate signature for authenticated requests
    fn generate_signature(&self, params: &str) -> String {
        use sha1::{Sha1, Digest};
        let to_sign = format!("{}{}", params, self.api_secret);
        let mut hasher = Sha1::new();
        hasher.update(to_sign.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

#[cfg(feature = "cloudinary")]
#[async_trait]
impl StorageProvider for CloudinaryStorage {
    async fn upload(
        &self,
        filename: &str,
        _content_type: &str,
        data: Vec<u8>,
        user_id: Uuid,
    ) -> Result<String> {
        // Public ID with user folder structure
        let public_id = format!("users/{}/{}", user_id, filename);
        
        // Create multipart form
        let file_part = Part::bytes(data)
            .file_name(filename.to_string())
            .mime_str("application/octet-stream")
            .map_err(|e| ApiError::InternalError(format!("Failed to create file part: {}", e)))?;
        
        let form = Form::new()
            .text("public_id", public_id.clone())
            .text("api_key", self.api_key.clone())
            .part("file", file_part);
        
        // Upload to Cloudinary
        let response = self.client
            .post(&self.get_upload_url())
            .multipart(form)
            .send()
            .await
            .map_err(|e| ApiError::InternalError(format!("Cloudinary upload request failed: {}", e)))?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(ApiError::InternalError(format!("Cloudinary upload failed: {}", error_text)));
        }
        
        let upload_response: CloudinaryUploadResponse = response
            .json()
            .await
            .map_err(|e| ApiError::InternalError(format!("Failed to parse Cloudinary response: {}", e)))?;
        
        tracing::info!("Uploaded file to Cloudinary: {}", upload_response.secure_url);
        
        // Return public_id as the key
        Ok(upload_response.public_id)
    }

    async fn download(&self, key: &str) -> Result<Vec<u8>> {
        // For Cloudinary, we download from the public URL
        let url = self.get_url(key);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| ApiError::InternalError(format!("Cloudinary download failed: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(ApiError::NotFound(format!("File not found: {}", key)));
        }
        
        let data = response
            .bytes()
            .await
            .map_err(|e| ApiError::InternalError(format!("Failed to read file data: {}", e)))?;
        
        Ok(data.to_vec())
    }

    async fn delete(&self, key: &str) -> Result<()> {
        // Determine resource type from public_id
        let resource_type = if key.contains("/video/") || key.ends_with(".mp4") || key.ends_with(".mov") {
            "video"
        } else if key.contains("/raw/") {
            "raw"
        } else {
            "image"
        };
        
        // Generate signature
        let timestamp = chrono::Utc::now().timestamp();
        let params = format!("public_id={}&timestamp={}", key, timestamp);
        let signature = self.generate_signature(&params);
        
        // Delete from Cloudinary
        let form = Form::new()
            .text("public_id", key.to_string())
            .text("api_key", self.api_key.clone())
            .text("timestamp", timestamp.to_string())
            .text("signature", signature);
        
        let response = self.client
            .post(&self.get_delete_url(resource_type))
            .multipart(form)
            .send()
            .await
            .map_err(|e| ApiError::InternalError(format!("Cloudinary delete request failed: {}", e)))?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(ApiError::InternalError(format!("Cloudinary delete failed: {}", error_text)));
        }
        
        tracing::info!("Deleted file from Cloudinary: {}", key);
        
        Ok(())
    }

    fn get_url(&self, key: &str) -> String {
        // Cloudinary URL format
        format!(
            "https://res.cloudinary.com/{}/image/upload/{}",
            self.cloud_name, key
        )
    }

    async fn generate_presigned_url(
        &self,
        key: &str,
        expires_in: std::time::Duration,
    ) -> Result<String> {
        use sha1::{Sha1, Digest};
        use chrono::Utc;
        
        // Cloudinary uses authenticated URLs with expiration timestamp
        let expires_at = Utc::now().timestamp() + expires_in.as_secs() as i64;
        
        // Create the string to sign: resource_type + type + version + public_id
        let to_sign = format!("timestamp={}&public_id={}{}", expires_at, key, self.api_secret);
        
        let mut hasher = Sha1::new();
        hasher.update(to_sign.as_bytes());
        let signature = format!("{:x}", hasher.finalize());
        
        // Build authenticated URL
        let url = format!(
            "https://res.cloudinary.com/{}/image/authenticated/s--{}--/{}?timestamp={}",
            self.cloud_name,
            signature,
            key,
            expires_at
        );
        
        tracing::info!("Generated Cloudinary presigned URL for: {}", key);
        
        Ok(url)
    }

    fn backend_name(&self) -> &str {
        "cloudinary"
    }
}

#[cfg(feature = "cloudinary")]
impl CloudinaryStorage {
    /// Get thumbnail URL with transformations
    pub fn get_thumbnail_url(&self, key: &str, width: u32, height: u32) -> String {
        format!(
            "https://res.cloudinary.com/{}/image/upload/w_{},h_{},c_fill/{}",
            self.cloud_name, width, height, key
        )
    }

    /// Get transformed URL with custom transformations
    pub fn get_transformed_url(&self, key: &str, transformation: &str) -> String {
        format!(
            "https://res.cloudinary.com/{}/image/upload/{}/{}",
            self.cloud_name, transformation, key
        )
    }
}

