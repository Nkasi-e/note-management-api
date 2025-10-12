use axum::{
    routing::{get, post, delete},
    Router,
};

use crate::handlers::{
    upload_file, download_file, get_file_metadata,
    list_user_files, delete_file, get_user_stats,
    generate_presigned_url,
};
use crate::services::FileService;
use crate::middleware::auth_middleware;
use crate::config::settings::AuthConfig;

/// File routes
pub fn file_routes(file_service: FileService, auth_config: AuthConfig) -> Router {
    Router::new()
        // Protected routes (require authentication)
        // Note: More specific routes MUST come before less specific ones
        .route("/stats", get(get_user_stats))
        .route("/", post(upload_file))
        .route("/", get(list_user_files))
        .route("/:file_id/presigned-url", get(generate_presigned_url))
        .route("/:file_id/download", get(download_file))
        .route("/:file_id/metadata", get(get_file_metadata))
        .route("/:file_id", delete(delete_file))
        
        // Apply authentication middleware to all routes
        .layer(axum::middleware::from_fn_with_state(
            auth_config,
            auth_middleware
        ))
        
        .with_state(file_service)
}

