use axum::{
    extract::{Extension, Path},
    response::Json,
};
use uuid::Uuid;
use serde_json::{json};
use tracing::{info, debug};

use crate::{
    domain::error::ApiError,
    handlers::api_response::respond_ok,
    workers::{WorkerService, JobType, JobPriority, EmailJobPayload, TaskReminderPayload, CleanupJobPayload, ReportJobPayload, AttachmentJobPayload},
    middleware::CurrentUser,
};

/// Enqueue an email job
pub async fn enqueue_email_job(
    Extension(worker_service): Extension<WorkerService>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<EmailJobPayload>,
) -> Result<Json<serde_json::Value>, ApiError> {
    info!("Enqueuing email job for user {}", current_user.id);
    debug!("Email job payload: {:?}", payload);

    let job_id = worker_service
        .enqueue_job(JobType::SendEmail(payload), JobPriority::Normal)
        .await
        .map_err(|e| {
            tracing::error!("Failed to enqueue email job: {}", e);
            ApiError::internal_error("Failed to enqueue email job")
        })?;

    Ok(respond_ok(json!({
        "job_id": job_id,
        "message": "Email job enqueued successfully"
    })))
}

/// Enqueue a task reminder job
pub async fn enqueue_task_reminder(
    Extension(worker_service): Extension<WorkerService>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<TaskReminderPayload>,
) -> Result<Json<serde_json::Value>, ApiError> {
    info!("Enqueuing task reminder for user {}", current_user.id);
    debug!("Task reminder payload: {:?}", payload);

    let job_id = worker_service
        .enqueue_job(JobType::SendTaskReminder(payload), JobPriority::High)
        .await
        .map_err(|e| {
            tracing::error!("Failed to enqueue task reminder: {}", e);
            ApiError::internal_error("Failed to enqueue task reminder")
        })?;

    Ok(respond_ok(json!({
        "job_id": job_id,
        "message": "Task reminder enqueued successfully"
    })))
}

/// Enqueue a cleanup job
pub async fn enqueue_cleanup_job(
    Extension(worker_service): Extension<WorkerService>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<CleanupJobPayload>,
) -> Result<Json<serde_json::Value>, ApiError> {
    info!("Enqueuing cleanup job for user {}", current_user.id);
    debug!("Cleanup job payload: {:?}", payload);

    let job_id = worker_service
        .enqueue_job(JobType::CleanupTasks(payload), JobPriority::Low)
        .await
        .map_err(|e| {
            tracing::error!("Failed to enqueue cleanup job: {}", e);
            ApiError::internal_error("Failed to enqueue cleanup job")
        })?;

    Ok(respond_ok(json!({
        "job_id": job_id,
        "message": "Cleanup job enqueued successfully"
    })))
}

/// Enqueue a report generation job
pub async fn enqueue_report_job(
    Extension(worker_service): Extension<WorkerService>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<ReportJobPayload>,
) -> Result<Json<serde_json::Value>, ApiError> {
    info!("Enqueuing report job for user {}", current_user.id);
    debug!("Report job payload: {:?}", payload);

    let job_id = worker_service
        .enqueue_job(JobType::GenerateReport(payload), JobPriority::Normal)
        .await
        .map_err(|e| {
            tracing::error!("Failed to enqueue report job: {}", e);
            ApiError::internal_error("Failed to enqueue report job")
        })?;

    Ok(respond_ok(json!({
        "job_id": job_id,
        "message": "Report job enqueued successfully"
    })))
}

/// Enqueue an attachment processing job
pub async fn enqueue_attachment_job(
    Extension(worker_service): Extension<WorkerService>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<AttachmentJobPayload>,
) -> Result<Json<serde_json::Value>, ApiError> {
    info!("Enqueuing attachment job for user {}", current_user.id);
    debug!("Attachment job payload: {:?}", payload);

    let job_id = worker_service
        .enqueue_job(JobType::ProcessAttachment(payload), JobPriority::Normal)
        .await
        .map_err(|e| {
            tracing::error!("Failed to enqueue attachment job: {}", e);
            ApiError::internal_error("Failed to enqueue attachment job")
        })?;

    Ok(respond_ok(json!({
        "job_id": job_id,
        "message": "Attachment job enqueued successfully"
    })))
}

/// Get job status
pub async fn get_job_status(
    Extension(worker_service): Extension<WorkerService>,
    Extension(current_user): Extension<CurrentUser>,
    Path(job_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    info!("Getting job status for job {} by user {}", job_id, current_user.id);

    let job_metadata = worker_service
        .get_job_status(job_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get job status: {}", e);
            ApiError::internal_error("Failed to get job status")
        })?;

    match job_metadata {
        Some(metadata) => Ok(respond_ok(json!({
            "job_id": metadata.id,
            "status": metadata.status,
            "created_at": metadata.created_at,
            "started_at": metadata.started_at,
            "completed_at": metadata.completed_at,
            "retry_count": metadata.retry_count,
            "max_retries": metadata.max_retries,
            "error_message": metadata.error_message,
            "priority": metadata.priority
        }))),
        None => Err(ApiError::not_found("Job not found")),
    }
}

/// Get queue statistics
pub async fn get_queue_stats(
    Extension(worker_service): Extension<WorkerService>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<serde_json::Value>, ApiError> {
    info!("Getting queue stats for user {}", current_user.id);

    let stats = worker_service
        .get_queue_stats()
        .await
        .map_err(|e| {
            tracing::error!("Failed to get queue stats: {}", e);
            ApiError::internal_error("Failed to get queue stats")
        })?;

    Ok(respond_ok(json!({
        "pending_jobs": stats.pending_jobs,
        "worker_pool_size": stats.worker_pool_size,
        "is_running": stats.is_running
    })))
}
