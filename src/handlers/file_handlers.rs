use axum::{
    extract::{Path, State, Multipart, Extension},
    response::{Response, IntoResponse},
    http::{header, StatusCode},
    body::Body,
};
use uuid::Uuid;
use tracing::info;

use crate::domain::{Result, ApiError};
use crate::services::FileService;
use crate::middleware::CurrentUser;
use super::{respond_ok, respond_created};

/// Upload a file
///
/// Uploads a file to the configured storage backend (local, S3, GCS, Azure, or Cloudinary).
/// The file is stored with a unique filename and associated with the authenticated user.
/// Maximum file size: 10MB (configurable).
#[utoipa::path(
    post,
    path = "/api/v1/files/upload",
    tag = "files",
    request_body(
        content = inline(String),
        content_type = "multipart/form-data",
        description = "File to upload"
    ),
    responses(
        (status = 201, description = "File uploaded successfully", body = inline(serde_json::Value)),
        (status = 400, description = "Invalid file or missing data", body = ApiErrorResponse),
        (status = 413, description = "File too large", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 500, description = "Storage error", body = ApiErrorResponse),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn upload_file(
    State(file_service): State<FileService>,
    Extension(current_user): Extension<CurrentUser>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse> {
    info!("User {} uploading file", current_user.id);

    let mut file_data = None;
    let mut filename = None;
    let mut content_type = None;

    // Parse multipart form data
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        ApiError::bad_request(format!("Failed to parse multipart data: {}", e))
    })? {
        let field_name = field.name().unwrap_or("").to_string();

        match field_name.as_str() {
            "file" => {
                filename = field.file_name().map(|s| s.to_string());
                content_type = field.content_type().map(|s| s.to_string());
                
                // field.bytes() returns Bytes directly - zero-copy!
                let data = field.bytes().await.map_err(|e| {
                    ApiError::bad_request(format!("Failed to read file data: {}", e))
                })?;
                
                file_data = Some(data);  // ← Now storing Bytes, not Vec<u8>
            }
            _ => {
                // Ignore unknown fields
            }
        }
    }

    // Validate required fields
    let file_data = file_data.ok_or_else(|| {
        ApiError::bad_request("No file provided in the request")
    })?;

    let filename = filename.ok_or_else(|| {
        ApiError::bad_request("No filename provided")
    })?;

    let content_type = content_type.unwrap_or_else(|| "application/octet-stream".to_string());

    // Upload the file
    let file_metadata = file_service
        .upload_file(
            filename,
            content_type,
            file_data,
            current_user.id,
        )
        .await?;

    info!("File uploaded successfully: {}", file_metadata.id);

    // Return file metadata with download URL
    Ok(respond_created(serde_json::json!({
        "id": file_metadata.id,
        "filename": file_metadata.original_filename,
        "content_type": file_metadata.content_type,
        "size": file_metadata.size,
        "url": format!("/api/v1/files/{}/download", file_metadata.id),
        "created_at": file_metadata.created_at,
    })))
}

/// Download a file
///
/// Downloads a file by its ID. Uses zero-copy streaming for efficient memory handling.
/// Only the file owner can download their files. Returns the file with appropriate
/// Content-Type and Content-Disposition headers.
#[utoipa::path(
    get,
    path = "/api/v1/files/{id}/download",
    tag = "files",
    params(
        ("id" = Uuid, Path, description = "File ID")
    ),
    responses(
        (status = 200, description = "File downloaded successfully", content_type = "application/octet-stream"),
        (status = 404, description = "File not found", body = ApiErrorResponse),
        (status = 403, description = "Forbidden - not your file", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn download_file(
    State(file_service): State<FileService>,
    Extension(current_user): Extension<CurrentUser>,
    Path(file_id): Path<Uuid>,
) -> Result<Response> {
    info!("User {} downloading file {}", current_user.id, file_id);

    // Get metadata first to get the filename
    let metadata = file_service.get_file_metadata(file_id).await?;
    
    // Check ownership
    if metadata.uploaded_by != current_user.id {
        return Err(ApiError::Forbidden("You don't have permission to access this file".to_string()));
    }

    // Download file - returns Bytes (zero-copy!)
    let (metadata, file_data) = file_service
        .download_file(&metadata.filename)
        .await?;

    // Create body from Bytes - no copy, just reference count increment
    let body = Body::from(file_data);

    // Build response with appropriate headers
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, &metadata.content_type)
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", metadata.original_filename),
        )
        .header(header::CONTENT_LENGTH, metadata.size.to_string())
        .body(body)
        .map_err(|e| ApiError::InternalError(format!("Failed to build response: {}", e)))?;

    Ok(response)
}

/// Delete a file
///
/// Deletes a file by its ID. Only the file owner can delete their files.
/// This removes both the database record and the file from storage.
#[utoipa::path(
    delete,
    path = "/api/v1/files/{id}",
    tag = "files",
    params(
        ("id" = Uuid, Path, description = "File ID")
    ),
    responses(
        (status = 200, description = "File deleted successfully", body = inline(serde_json::Value)),
        (status = 404, description = "File not found", body = ApiErrorResponse),
        (status = 403, description = "Forbidden - not your file", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_file(
    State(file_service): State<FileService>,
    Extension(current_user): Extension<CurrentUser>,
    Path(file_id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    info!("User {} deleting file {}", current_user.id, file_id);

    file_service.delete_file(file_id, current_user.id).await?;

    Ok(respond_ok(serde_json::json!({
        "message": "File deleted successfully"
    })))
}

/// Get file metadata
///
/// Retrieves metadata for a specific file by its ID. Only the file owner
/// can access their file metadata. Returns information like filename, size,
/// content type, and upload date.
#[utoipa::path(
    get,
    path = "/api/v1/files/{id}",
    tag = "files",
    params(
        ("id" = Uuid, Path, description = "File ID")
    ),
    responses(
        (status = 200, description = "File metadata retrieved successfully", body = FileMetadata),
        (status = 404, description = "File not found", body = ApiErrorResponse),
        (status = 403, description = "Forbidden - not your file", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_file_metadata(
    State(file_service): State<FileService>,
    Extension(current_user): Extension<CurrentUser>,
    Path(file_id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    info!("User {} getting metadata for file {}", current_user.id, file_id);

    let metadata = file_service.get_file_metadata(file_id).await?;
    
    // Check ownership
    if metadata.uploaded_by != current_user.id {
        return Err(ApiError::Forbidden("You don't have permission to access this file".to_string()));
    }

    Ok(respond_ok(metadata))
}

/// List user's files
///
/// Returns a list of all files uploaded by the authenticated user.
/// Includes metadata for each file such as filename, size, content type, and upload date.
#[utoipa::path(
    get,
    path = "/api/v1/files",
    tag = "files",
    responses(
        (status = 200, description = "Files retrieved successfully", body = Vec<FileMetadata>),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_user_files(
    State(file_service): State<FileService>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<impl IntoResponse> {
    info!("User {} listing files", current_user.id);

    let files = file_service.list_user_files(current_user.id).await?;

    Ok(respond_ok(files))
}

/// Get file statistics
///
/// Returns statistics about the authenticated user's file uploads, including
/// total number of files, total storage used (in bytes and MB).
#[utoipa::path(
    get,
    path = "/api/v1/files/stats",
    tag = "files",
    responses(
        (status = 200, description = "Statistics retrieved successfully", body = FileStats),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_user_stats(
    State(file_service): State<FileService>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<impl IntoResponse> {
    info!("User {} getting file stats", current_user.id);

    let stats = file_service.get_user_stats(current_user.id).await?;

    Ok(respond_ok(stats))
}

/// Generate presigned URL
///
/// Generates a temporary presigned URL for secure file access without authentication.
/// The URL expires after 1 hour (3600 seconds). Useful for:
/// - Sharing private files securely
/// - Direct browser downloads
/// - Granting time-limited access to external services
///
/// Only available for cloud storage providers (S3, GCS, Azure, Cloudinary).
#[utoipa::path(
    get,
    path = "/api/v1/files/{id}/presigned-url",
    tag = "files",
    params(
        ("id" = Uuid, Path, description = "File ID")
    ),
    responses(
        (status = 200, description = "Presigned URL generated successfully", body = inline(serde_json::Value)),
        (status = 404, description = "File not found", body = ApiErrorResponse),
        (status = 403, description = "Forbidden - not your file", body = ApiErrorResponse),
        (status = 501, description = "Not supported by current storage provider", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn generate_presigned_url(
    Path(file_id): Path<Uuid>,
    State(file_service): State<FileService>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<impl IntoResponse> {
    info!("User {} generating presigned URL for file {}", current_user.id, file_id);

    // Default expiration: 1 hour (3600 seconds)
    // In production, you might want to accept this as a query parameter
    let expires_in = std::time::Duration::from_secs(3600);

    let presigned_url = file_service
        .generate_presigned_url(file_id, current_user.id, expires_in)
        .await?;

    Ok(respond_ok(serde_json::json!({
        "url": presigned_url,
        "expires_in_seconds": expires_in.as_secs(),
        "expires_at": chrono::Utc::now() + chrono::Duration::seconds(expires_in.as_secs() as i64),
    })))
}
