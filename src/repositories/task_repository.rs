use sqlx::PgPool;
use uuid::Uuid;
use crate::domain::{Task, CreateTaskRequest, Result, ApiError};
use crate::domain::task::{slugify, TaskStatus};

#[derive(Debug, Clone)]
pub struct TaskRepository {
    pool: PgPool,
}

impl TaskRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, request: CreateTaskRequest) -> Result<Task> {
        let rec = sqlx::query_as!(
            Task,
            r#"
            INSERT INTO tasks (title, description, slug, status, user_id)
            VALUES ($1, $2, $3, 'todo', $4)
            RETURNING 
              id, title, description, slug, 
              status as "status: TaskStatus", 
              user_id, created_at, updated_at
            "#,
            request.title,
            request.description,
            slugify(&request.title),
            request.user_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ApiError::InternalError(format!("DB insert task error: {}", e)))?;

        Ok(rec)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Task> {
        let rec = sqlx::query_as!(
            Task,
            r#"
            SELECT 
              id, title, description, slug, 
              status as "status: TaskStatus", 
              user_id, created_at, updated_at
            FROM tasks
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ApiError::InternalError(format!("DB select task error: {}", e)))?;

        rec.ok_or(ApiError::TaskNotFound { id })
    }

    pub async fn find_by_user_id(&self, user_id: Uuid) -> Result<Vec<Task>> {
        let recs = sqlx::query_as!(
            Task,
            r#"
            SELECT 
              id, title, description, slug, 
              status as "status: TaskStatus", 
              user_id, created_at, updated_at
            FROM tasks
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ApiError::InternalError(format!("DB select tasks by user error: {}", e)))?;

        Ok(recs)
    }

    pub async fn find_all(&self) -> Result<Vec<Task>> {
        let recs = sqlx::query_as!(
            Task,
            r#"
            SELECT 
              id, title, description, slug, 
              status as "status: TaskStatus", 
              user_id, created_at, updated_at
            FROM tasks
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ApiError::InternalError(format!("DB select all tasks error: {}", e)))?;

        Ok(recs)
    }

    pub async fn count(&self) -> usize {
        let rec: Result<Option<(i64,)>> = sqlx::query_as(
            "SELECT COUNT(*) FROM tasks"
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ApiError::InternalError(format!("DB count tasks error: {}", e)));

        rec.ok().flatten().map(|t| t.0 as usize).unwrap_or(0)
    }
}
