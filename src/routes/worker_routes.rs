use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

use crate::{
    handlers::{enqueue_email_job, enqueue_task_reminder, enqueue_cleanup_job, 
        enqueue_report_job, enqueue_attachment_job, get_job_status, get_queue_stats
    },
    workers::WorkerService,
    middleware::auth_middleware,
    config::settings::AuthConfig,
};

/// Worker-related routes
pub fn worker_routes(worker_service: Arc<WorkerService>, auth_config: AuthConfig) -> Router {
    Router::new()
        // Job enqueueing endpoints
        .route("/jobs/email", post(enqueue_email_job))
        .route("/jobs/reminder", post(enqueue_task_reminder))
        .route("/jobs/cleanup", post(enqueue_cleanup_job))
        .route("/jobs/report", post(enqueue_report_job))
        .route("/jobs/attachment", post(enqueue_attachment_job))
        
        // Job status and queue management
        .route("/jobs/:job_id/status", get(get_job_status))
        .route("/queue/stats", get(get_queue_stats))
        
        // Authentication middleware
        .layer(axum::middleware::from_fn_with_state(
            auth_config,
            auth_middleware
        ))
        .with_state(worker_service)
}
