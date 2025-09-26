use sqlx::PgPool;
use uuid::Uuid;
use crate::domain::{Task, Result, ApiError};
use crate::domain::task::{slugify, TaskStatus};

#[derive(Debug, Clone)]
pub struct CreateTaskRequestInternal {
    pub title: String,
    pub description: Option<String>,
    pub user_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct TaskRepository {
    pool: PgPool,
}

impl TaskRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, request: CreateTaskRequestInternal) -> Result<Task> {
        // Generate unique slug
        let slug = self.generate_unique_slug(&request.title).await?;
        
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
            slug,
            request.user_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ApiError::InternalError(format!("DB insert task error: {}", e)))?;

        Ok(rec)
    }

    async fn generate_unique_slug(&self, title: &str) -> Result<String> {
        let base_slug = slugify(title);
        let mut slug = base_slug.clone();
        let mut attempts = 0;

        loop {
            // Check if slug exists
            let exists = sqlx::query_scalar!(
                "SELECT EXISTS(SELECT 1 FROM tasks WHERE slug = $1)",
                slug
            )
            .fetch_one(&self.pool)
            .await
            .map_err(|e| ApiError::InternalError(format!("DB check slug error: {}", e)))?;

            if !exists.unwrap_or(false) {
                break;
            }

            // Generate new slug with 4-digit random string
            let random_suffix = generate_random_suffix();
            slug = format!("{}-{}", base_slug, random_suffix);
            attempts += 1;

            // Prevent infinite loop (safety check)
            if attempts > 1000 {
                return Err(ApiError::InternalError("Unable to generate unique slug".to_string()));
            }
        }

        Ok(slug)
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

/// Generate a 4-digit random string for slug uniqueness
fn generate_random_suffix() -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::{SystemTime, UNIX_EPOCH};
    
    // Use current timestamp + random seed for uniqueness
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    
    let mut hasher = DefaultHasher::new();
    timestamp.hash(&mut hasher);
    let hash = hasher.finish();
    
    // Convert to 4-digit string (base 36 for shorter strings)
    format!("{:04x}", (hash % 65536) as u16)
}
