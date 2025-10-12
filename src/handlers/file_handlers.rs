use axum::{
    extract::{Path, State, Multipart, Extension},
    response::{Response, IntoResponse},
    http::{header, StatusCode},
    body::Body,
};
use tokio_util::io::ReaderStream;
use uuid::Uuid;
use tracing::info;

use crate::domain::{Result, ApiError};
use crate::services::FileService;
use crate::middleware::CurrentUser;
use super::{respond_ok, respond_created};

/// Upload a file
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
                
                let data = field.bytes().await.map_err(|e| {
                    ApiError::bad_request(format!("Failed to read file data: {}", e))
                })?;
                
                file_data = Some(data.to_vec());
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

    // Get file stream
    let (metadata, file) = file_service
        .get_file_stream(&metadata.filename)
        .await?;

    // Convert the async read stream to a body
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

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
pub async fn list_user_files(
    State(file_service): State<FileService>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<impl IntoResponse> {
    info!("User {} listing files", current_user.id);

    let files = file_service.list_user_files(current_user.id).await?;

    Ok(respond_ok(files))
}

/// Get file statistics for the current user
pub async fn get_user_stats(
    State(file_service): State<FileService>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<impl IntoResponse> {
    info!("User {} getting file stats", current_user.id);

    let stats = file_service.get_user_stats(current_user.id).await?;

    Ok(respond_ok(stats))
}
