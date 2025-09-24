use sqlx::PgPool;
use uuid::Uuid;
use crate::domain::{User, CreateUserRequest, Result, ApiError};

#[derive(Debug, Clone)]
pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_with_password_hash(&self, name: String, email: String, password_hash: String) -> Result<User> {
        // Check if email exists
        let exists: Option<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM users WHERE email = $1"
        )
        .bind(&email)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ApiError::InternalError(format!("DB error checking email: {}", e)))?;

        if exists.is_some() {
            return Err(ApiError::EmailAlreadyExists { email });
        }

        let rec = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (name, email, password_hash)
            VALUES ($1, $2, $3)
            RETURNING id, name, email, role as "role: crate::domain::user::UserRole", created_at
            "#,
            name,
            email,
            password_hash
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ApiError::InternalError(format!("DB insert user error: {}", e)))?;

        Ok(rec)
    }

    pub async fn find_auth_by_email(&self, email: &str) -> std::result::Result<Option<(User, String)>, sqlx::Error> {
        let rec = sqlx::query!(
            r#"
            SELECT id, name, email, role as "role: crate::domain::user::UserRole", created_at, password_hash
            FROM users
            WHERE email = $1
            "#,
            email
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(rec.map(|r| (User { id: r.id, name: r.name, email: r.email, role: r.role, created_at: r.created_at }, r.password_hash)))
    }

    pub async fn create(&self, request: CreateUserRequest) -> Result<User> {
        // Check if email exists
        let exists: Option<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM users WHERE email = $1"
        )
        .bind(&request.email)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ApiError::InternalError(format!("DB error checking email: {}", e)))?;

        if exists.is_some() {
            return Err(ApiError::EmailAlreadyExists { email: request.email });
        }

        let rec = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (name, email, password_hash)
            VALUES ($1, $2, '')
            RETURNING id, name, email, role as "role: crate::domain::user::UserRole", created_at
            "#,
            request.name,
            request.email
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ApiError::InternalError(format!("DB insert user error: {}", e)))?;

        Ok(rec)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<User> {
        let rec = sqlx::query_as!(
            User,
            r#"
            SELECT id, name, email, role as "role: crate::domain::user::UserRole", created_at
            FROM users
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ApiError::InternalError(format!("DB select user error: {}", e)))?;

        rec.ok_or(ApiError::UserNotFound { id })
    }

    pub async fn exists(&self, id: Uuid) -> bool {
        let rec: Result<Option<(Uuid,)>> = sqlx::query_as(
            "SELECT id FROM users WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ApiError::InternalError(format!("DB exists user error: {}", e)));

        matches!(rec, Ok(Some(_)))
    }

    pub async fn count(&self) -> usize {
        let rec: Result<Option<(i64,)>> = sqlx::query_as(
            "SELECT COUNT(*) FROM users"
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ApiError::InternalError(format!("DB count users error: {}", e)));

        rec.ok().flatten().map(|t| t.0 as usize).unwrap_or(0)
    }
}
