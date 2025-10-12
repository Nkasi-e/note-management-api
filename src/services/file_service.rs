use std::path::{Path, PathBuf};
use tokio::fs::{self, File};
use uuid::Uuid;
use chrono::Utc;
use tracing::{info, debug};
use std::sync::Arc;

use crate::config::settings::StorageConfig;
use crate::domain::{ApiError, Result, FileMetadata, UploadFileResponse, FileStats};
use crate::repositories::FileRepository;
use crate::storage::{StorageProvider, StorageFactory};

#[derive(Clone)]
pub struct FileService {
    repository: FileRepository,
    config: StorageConfig,
    storage: Arc<Box<dyn StorageProvider>>,
}

impl FileService {
    pub async fn new(repository: FileRepository, config: StorageConfig) -> Result<Self> {
        let storage = StorageFactory::create(&config).await?;
        Ok(Self { 
            repository, 
            config,
            storage: Arc::new(storage),
        })
    }

    /// Initialize storage directory
    pub async fn init_storage(&self) -> Result<()> {
        let upload_dir = Path::new(&self.config.upload_dir);
        
        if !upload_dir.exists() {
            fs::create_dir_all(upload_dir)
                .await
                .map_err(|e| ApiError::InternalError(format!("Failed to create upload directory: {}", e)))?;
            info!("Created upload directory: {}", self.config.upload_dir);
        }
        
        Ok(())
    }

    /// Upload a file with streaming
    pub async fn upload_file(
        &self,
        filename: String,
        content_type: String,
        data: Vec<u8>,
        user_id: Uuid,
    ) -> Result<UploadFileResponse> {
        // Validate file size
        if data.len() > self.config.max_file_size {
            return Err(ApiError::BadRequest(format!(
                "File size exceeds maximum allowed size of {} bytes",
                self.config.max_file_size
            )));
        }

        // Validate file extension
        let extension = Path::new(&filename)
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .unwrap_or_default();

        if !self.config.allowed_extensions.contains(&extension) {
            return Err(ApiError::BadRequest(format!(
                "File type '{}' is not allowed. Allowed types: {}",
                extension,
                self.config.allowed_extensions.join(", ")
            )));
        }

        // Generate unique filename
        let unique_filename = self.generate_unique_filename(&filename);
        
        // Upload using storage provider
        let storage_key = self.storage.upload(
            &unique_filename,
            &content_type,
            data.clone(),
            user_id,
        ).await?;

        info!("File uploaded to {} storage: {}", self.storage.backend_name(), storage_key);

        // Save metadata to database
        let metadata = FileMetadata {
            id: Uuid::new_v4(),
            filename: unique_filename.clone(),
            original_filename: filename,
            content_type,
            size: data.len() as i64,
            path: storage_key, // Store storage key (local path or S3 key)
            uploaded_by: user_id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let saved_metadata = self.repository.create(metadata).await?;
        
        info!("File metadata saved to database: {}", saved_metadata.id);

        Ok(saved_metadata.into())
    }

    /// Download a file with streaming
    pub async fn download_file(&self, filename: &str) -> Result<(FileMetadata, Vec<u8>)> {
        // Get metadata from database
        let metadata = self.repository.find_by_filename(filename).await?;

        // Download from storage provider
        let contents = self.storage.download(&metadata.path).await?;

        debug!("File read from {} storage: {} ({} bytes)", 
               self.storage.backend_name(), filename, contents.len());

        Ok((metadata, contents))
    }

    /// Stream file for download (more efficient for large files)
    pub async fn get_file_stream(&self, filename: &str) -> Result<(FileMetadata, File)> {
        // Get metadata from database
        let metadata = self.repository.find_by_filename(filename).await?;

        // Open file for streaming
        let file_path = Path::new(&metadata.path);
        
        if !file_path.exists() {
            return Err(ApiError::NotFound(format!("File {} not found on disk", filename)));
        }

        let file = File::open(file_path)
            .await
            .map_err(|e| ApiError::InternalError(format!("Failed to open file: {}", e)))?;

        Ok((metadata, file))
    }

    /// Delete a file
    pub async fn delete_file(&self, file_id: Uuid, user_id: Uuid) -> Result<()> {
        // Check ownership
        let is_owner = self.repository.is_owner(file_id, user_id).await?;
        if !is_owner {
            return Err(ApiError::Forbidden("You don't have permission to delete this file".to_string()));
        }

        // Get metadata
        let metadata = self.repository.find_by_id(file_id).await?;

        // Delete from storage provider
        self.storage.delete(&metadata.path).await?;
        info!("File deleted from {} storage: {}", self.storage.backend_name(), metadata.path);

        // Delete from database
        self.repository.delete(file_id).await?;
        
        info!("File metadata deleted from database: {}", file_id);

        Ok(())
    }

    /// List user's files
    pub async fn list_user_files(&self, user_id: Uuid) -> Result<Vec<UploadFileResponse>> {
        let files = self.repository.find_by_user(user_id).await?;
        Ok(files.into_iter().map(|f| f.into()).collect())
    }

    /// Get user file statistics
    pub async fn get_user_stats(&self, user_id: Uuid) -> Result<FileStats> {
        self.repository.get_user_stats(user_id).await
    }

    /// Get file metadata by ID
    pub async fn get_file_metadata(&self, file_id: Uuid) -> Result<FileMetadata> {
        self.repository.find_by_id(file_id).await
    }

    /// Generate a presigned URL for temporary file access
    /// This is useful for:
    /// - Sharing private files securely
    /// - Direct browser uploads/downloads without going through your server
    /// - Granting time-limited access to external services
    pub async fn generate_presigned_url(
        &self,
        file_id: Uuid,
        user_id: Uuid,
        expires_in: std::time::Duration,
    ) -> Result<String> {
        // Verify file ownership
        let is_owner = self.repository.is_owner(file_id, user_id).await?;
        if !is_owner {
            return Err(ApiError::Forbidden(
                "You don't have permission to access this file".to_string()
            ));
        }

        // Get file metadata
        let metadata = self.repository.find_by_id(file_id).await?;

        // For local storage, return the download URL with file_id
        if self.storage.backend_name() == "local" {
            let url = format!("/api/v1/files/{}/download", file_id);
            info!("Generated local download URL for file {}", file_id);
            return Ok(url);
        }

        // For cloud storage, generate presigned URL using the storage provider
        let presigned_url = self.storage
            .generate_presigned_url(&metadata.path, expires_in)
            .await?;

        info!(
            "Generated presigned URL for file {} (expires in {} seconds)",
            file_id,
            expires_in.as_secs()
        );

        Ok(presigned_url)
    }

    /// Generate unique filename with UUID prefix
    fn generate_unique_filename(&self, original: &str) -> String {
        let uuid = Uuid::new_v4();
        let extension = Path::new(original)
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| format!(".{}", e))
            .unwrap_or_default();
        
        format!("{}{}", uuid, extension)
    }

    /// Get full file path
    fn get_file_path(&self, filename: &str) -> PathBuf {
        Path::new(&self.config.upload_dir).join(filename)
    }

    /// Validate file before upload (extensible for virus scanning, etc.)
    pub fn validate_file(&self, filename: &str, size: usize) -> Result<()> {
        // Size validation
        if size > self.config.max_file_size {
            return Err(ApiError::BadRequest(format!(
                "File size {} exceeds maximum allowed size of {} bytes",
                size, self.config.max_file_size
            )));
        }

        // Extension validation
        let extension = Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .unwrap_or_default();

        if !self.config.allowed_extensions.contains(&extension) {
            return Err(ApiError::BadRequest(format!(
                "File type '{}' is not allowed",
                extension
            )));
        }

        // TODO: Add virus scanning here
        // self.scan_for_viruses(&data)?;

        Ok(())
    }

    /// Calculate file hash (for deduplication or integrity checks)
    pub fn calculate_hash(data: &[u8]) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

