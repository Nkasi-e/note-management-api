use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Different types of background jobs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum JobType {
    /// Send email notification
    SendEmail(EmailJobPayload),
    /// Send task reminder
    SendTaskReminder(TaskReminderPayload),
    /// Clean up old tasks
    CleanupTasks(CleanupJobPayload),
    /// Generate task report
    GenerateReport(ReportJobPayload),
    /// Process task attachments
    ProcessAttachment(AttachmentJobPayload),
}

/// Email notification job payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailJobPayload {
    pub to: String,
    pub subject: String,
    pub body: String,
    pub user_id: Uuid,
}

/// Task reminder job payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskReminderPayload {
    pub task_id: Uuid,
    pub user_id: Uuid,
    pub reminder_type: ReminderType,
    pub scheduled_for: DateTime<Utc>,
}

/// Types of task reminders
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReminderType {
    DueSoon,
    Overdue,
    Created,
    Completed,
}

/// Cleanup job payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupJobPayload {
    pub older_than_days: u32,
    pub status: Option<String>,
}

/// Report generation job payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportJobPayload {
    pub user_id: Uuid,
    pub report_type: ReportType,
    pub date_range: DateRange,
}

/// Types of reports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReportType {
    Weekly,
    Monthly,
    Yearly,
    Custom,
}

/// Date range for reports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

/// Attachment processing job payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentJobPayload {
    pub task_id: Uuid,
    pub attachment_path: String,
    pub processing_type: AttachmentProcessingType,
}

/// Types of attachment processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttachmentProcessingType {
    ImageResize { width: u32, height: u32 },
    PdfExtract,
    VideoThumbnail,
    DocumentIndex,
}

/// Job status tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Retrying,
}

/// Job metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobMetadata {
    pub id: Uuid,
    pub job_type: JobType,
    pub status: JobStatus,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub retry_count: u32,
    pub max_retries: u32,
    pub error_message: Option<String>,
    pub priority: JobPriority,
}

/// Job priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum JobPriority {
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
}

impl Default for JobPriority {
    fn default() -> Self {
        JobPriority::Normal
    }
}

/// Job queue configuration
#[derive(Debug, Clone)]
pub struct JobQueueConfig {
    pub max_retries: u32,
    pub retry_delay_secs: u64,
    pub job_timeout_secs: u64,
    pub worker_pool_size: usize,
    pub queue_name: String,
}

impl Default for JobQueueConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            retry_delay_secs: 60,
            job_timeout_secs: 300,
            worker_pool_size: 4,
            queue_name: "default".to_string(),
        }
    }
}
