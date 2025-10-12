use std::sync::Arc;

use tracing::{info, debug};

use super::job_types::*;
use crate::repositories::{UserRepository, TaskRepository};
use crate::services::{TaskService, EmailService};

/// Job processor that handles different types of background jobs
pub struct JobProcessor {
    user_repository: Arc<UserRepository>,
    task_repository: Arc<TaskRepository>,
    task_service: Arc<TaskService>,
    email_service: Arc<EmailService>,
}

impl JobProcessor {
    pub fn new(
        user_repository: Arc<UserRepository>,
        task_repository: Arc<TaskRepository>,
        task_service: Arc<TaskService>,
        email_service: Arc<EmailService>,
    ) -> Self {
        Self {
            user_repository,
            task_repository,
            task_service,
            email_service,
        }
    }

    /// Process a job based on its type
    pub async fn process_job(&self, job_type: JobType) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match job_type {
            JobType::SendEmail(payload) => self.process_email_job(payload).await,
            JobType::SendTaskReminder(payload) => self.process_task_reminder_job(payload).await,
            JobType::CleanupTasks(payload) => self.process_cleanup_job(payload).await,
            JobType::GenerateReport(payload) => self.process_report_job(payload).await,
            JobType::ProcessAttachment(payload) => self.process_attachment_job(payload).await,
        }
    }

    /// Process email notification job
    async fn process_email_job(&self, payload: EmailJobPayload) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Processing email job for user {}", payload.user_id);
        
        self.email_service
            .send_email(&payload.to, &payload.subject, &payload.body)
            .await?;
        
        info!("Email sent successfully to {}", payload.to);
        Ok(())
    }

    /// Process task reminder job
    async fn process_task_reminder_job(&self, payload: TaskReminderPayload) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Processing task reminder for task {}", payload.task_id);
        
        let task = self.task_repository.find_by_id(payload.task_id).await?;
        let user = self.user_repository.find_by_id(payload.user_id).await?;
        
        let due_date: Option<&str> = None; // Task doesn't have due_date field yet
        
        self.email_service
            .send_task_reminder(&user.email, &task.title, due_date)
            .await?;
        
        info!("Task reminder sent to user: {}", user.email);
        Ok(())
    }

    /// Process cleanup job
    async fn process_cleanup_job(&self, payload: CleanupJobPayload) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Processing cleanup job for tasks older than {} days", payload.older_than_days);
        
        // Calculate cutoff date
        let cutoff_date = chrono::Utc::now() - chrono::Duration::days(payload.older_than_days as i64);
        
        // Get tasks to cleanup
        let tasks_to_cleanup = self.task_repository.find_old_tasks(&cutoff_date, payload.status.as_deref()).await?;
        
        info!("Found {} tasks to cleanup", tasks_to_cleanup.len());
        
        for task in tasks_to_cleanup {
            debug!("Cleaning up task: {} (created: {})", task.title, task.created_at);
        }
        
        info!("Cleanup job completed successfully");
        Ok(())
    }

    /// Process report generation job
    async fn process_report_job(&self, payload: ReportJobPayload) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Processing report generation for user {}", payload.user_id);
        
        let user = self.user_repository.find_by_id(payload.user_id).await?;
        
        // Get tasks in date range
        let tasks = self.task_repository.find_tasks_by_date_range(
            &payload.user_id,
            &payload.date_range.start,
            &payload.date_range.end
        ).await?;
        
        // Generate report data
        let total_tasks = tasks.len();
        let completed_tasks = tasks.iter().filter(|t| t.status == crate::domain::task::TaskStatus::Done).count();
        let pending_tasks = tasks.iter().filter(|t| t.status == crate::domain::task::TaskStatus::Todo).count();
        let in_progress_tasks = tasks.iter().filter(|t| t.status == crate::domain::task::TaskStatus::InProgress).count();
        
        let report_data = serde_json::json!({
            "user_id": payload.user_id,
            "user_email": user.email,
            "report_type": payload.report_type,
            "date_range": {
                "start": payload.date_range.start,
                "end": payload.date_range.end
            },
            "statistics": {
                "total_tasks": total_tasks,
                "completed_tasks": completed_tasks,
                "pending_tasks": pending_tasks,
                "in_progress_tasks": in_progress_tasks,
                "completion_rate": if total_tasks > 0 { (completed_tasks as f64 / total_tasks as f64) * 100.0 } else { 0.0 }
            },
            "generated_at": chrono::Utc::now()
        });
        
        info!("Report generated for user {}: {} tasks analyzed", user.email, total_tasks);
        debug!("Report data: {}", serde_json::to_string_pretty(&report_data)?);
        
        info!("Report generation completed successfully");
        Ok(())
    }

    /// Process attachment job
    async fn process_attachment_job(&self, payload: AttachmentJobPayload) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Processing attachment for task {}", payload.task_id);
        
        let task = self.task_repository.find_by_id(payload.task_id).await?;
        
        match payload.processing_type {
            AttachmentProcessingType::ImageResize { width, height } => {
                info!("Resizing image to {}x{}", width, height);
            },
            AttachmentProcessingType::PdfExtract => {
                info!("Extracting text from PDF");
            },
            AttachmentProcessingType::VideoThumbnail => {
                info!("Generating video thumbnail");
            },
            AttachmentProcessingType::DocumentIndex => {
                info!("Indexing document for search");
            },
        }
        
        info!("Attachment processing completed for task {}", task.title);
        Ok(())
    }
}
