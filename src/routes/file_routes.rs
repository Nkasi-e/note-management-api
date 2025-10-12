use axum::{
    routing::{get, post, delete},
    Router,
};

use crate::handlers::{
    upload_file, download_file, get_file_metadata,
    list_user_files, delete_file, get_user_stats,
};
use crate::services::FileService;
use crate::middleware::auth_middleware;
use crate::config::settings::AuthConfig;

/// File routes
pub fn file_routes(file_service: FileService, auth_config: AuthConfig) -> Router {
    Router::new()
        // Protected routes (require authentication)
        // Note: More specific routes must come before less specific ones
        .route("/files/stats", get(get_user_stats))
        .route("/files/:file_id/metadata", get(get_file_metadata))
        .route("/files/:file_id/download", get(download_file))
        .route("/files/:file_id", delete(delete_file))
        .route("/files", post(upload_file))
        .route("/files", get(list_user_files))
        
        // Apply authentication middleware to all routes
        .layer(axum::middleware::from_fn_with_state(
            auth_config,
            auth_middleware
        ))
        
        .with_state(file_service)
}

