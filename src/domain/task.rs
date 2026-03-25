use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;
// ============================================================================
// ToSchema - OpenAPI Documentation Trait
// ============================================================================
// The ToSchema derive macro from utoipa automatically generates OpenAPI
// schema definitions for this type. This allows it to be documented in
// Swagger UI with proper field types, descriptions, and examples.
use utoipa::ToSchema;

// ============================================================================
// TaskStatus Enum
// ============================================================================
// This enum represents the possible states of a task.
//
// Derives:
// - ToSchema: Makes this enum appear in the OpenAPI spec
// - The enum will show as a dropdown in Swagger UI with these three options
//
// Attributes:
// - #[serde(rename = "...")]: Controls how the enum is serialized to JSON
//   Example: TaskStatus::InProgress becomes "in_progress" in JSON
#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, ToSchema)]
#[sqlx(type_name = "task_status", rename_all = "snake_case")]
pub enum TaskStatus {
    #[serde(rename = "todo")]
    Todo,
    #[serde(rename = "in_progress")]
    InProgress,
    #[serde(rename = "done")]
    Done,
}

// ============================================================================
// Task Struct - Main Task Entity
// ============================================================================
// This struct represents a task in the system.
//
// ToSchema derive:
// - Automatically generates OpenAPI schema showing all fields
// - Field types are inferred (Uuid, String, Option<String>, DateTime, etc.)
// - In Swagger UI, this will show as an expandable object with all properties
//
// Example in Swagger UI:
// {
//   "id": "550e8400-e29b-41d4-a716-446655440000",
//   "title": "Buy groceries",
//   "description": "Get milk and eggs",
//   "slug": "buy-groceries",
//   "status": "todo",
//   "user_id": "123e4567-e89b-12d3-a456-426614174000",
//   "created_at": "2024-01-01T00:00:00Z",
//   "updated_at": "2024-01-01T00:00:00Z"
// }
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Task {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub slug: String,
    pub status: TaskStatus,
    pub user_id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

// ============================================================================
// TaskWithAttachments - Task with File References
// ============================================================================
// This struct extends Task with file attachment information.
//
// #[serde(flatten)]: Flattens the Task fields into this struct's JSON
// representation, so instead of {"task": {...}, "attachment_ids": [...]},
// you get all Task fields at the top level plus attachment_ids.
//
// ToSchema: Documents this composite structure in OpenAPI
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TaskWithAttachments {
    #[serde(flatten)]
    pub task: Task,
    pub attachment_ids: Vec<Uuid>,
}

// ============================================================================
// CreateTaskRequest - Request Body for Creating Tasks
// ============================================================================
// This struct defines what data is required to create a new task.
//
// ToSchema: Makes this appear in Swagger UI as a request body schema
// When you add #[utoipa::path] to the create_task handler, you'll reference
// this struct, and Swagger UI will show users what fields they need to send.
//
// Example request body in Swagger UI:
// {
//   "title": "Buy groceries",
//   "description": "Get milk and eggs",
//   "attachment_ids": ["550e8400-e29b-41d4-a716-446655440000"]
// }
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateTaskRequest {
    pub title: String,
    pub description: Option<String>,
    pub attachment_ids: Option<Vec<Uuid>>,
}

impl Task {
    pub fn new(title: String, description: Option<String>, user_id: Uuid) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Uuid::new_v4(),
            title: title.clone(),
            description,
            slug: slugify(&title),
            status: TaskStatus::Todo,
            user_id,
            created_at: now,
            updated_at: now,
        }
    }
}

pub fn slugify(title: &str) -> String {
    title
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| match c {
            'a'..='z' | '0'..='9' => c,
            ' ' => '-',
            _ => '-',
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}
