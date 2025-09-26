use axum::{
    extract::{Path, Query, State, Extension},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;
use tracing::{info, debug};

use crate::domain::{CreateTaskRequest, Result, ApiError};
use crate::domain::user::UserRole;
use crate::services::TaskService;
use crate::middleware::CurrentUser;
use super::{respond_created, respond_ok};

#[derive(Debug, Deserialize)]
pub struct TaskIdPath {
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct TasksQuery {
    pub user_id: Option<String>,
}

pub async fn create_task(
    State(task_service): State<TaskService>,
    Extension(current_user): Extension<CurrentUser>,
    Json(request): Json<CreateTaskRequest>,
) -> Result<impl IntoResponse> {
    info!("Creating task for user {}: {}", current_user.id, request.title);
    debug!("Task request payload: {:?}", request);
    
    let task = task_service.create_task(request, current_user.id).await?;
    
    info!("Task created successfully: {} (slug: {})", task.id, task.slug);
    Ok(respond_created(task))
}

pub async fn get_task(
    State(task_service): State<TaskService>,
    Extension(current_user): Extension<CurrentUser>,
    Path(params): Path<TaskIdPath>,
) -> Result<impl IntoResponse> {
    let task_id = params
        .id
        .parse::<Uuid>()
        .map_err(|_| ApiError::bad_request(format!("Invalid task ID format: {}", params.id)))?;

    let task = task_service.get_task(task_id).await?;

    // Users can only view their own tasks, admins can view any task
    if current_user.role != UserRole::Admin && current_user.id != task.user_id {
        return Err(ApiError::forbidden("You can only view your own tasks"));
    }

    Ok(respond_ok(task))
}

pub async fn get_tasks(
    State(task_service): State<TaskService>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<TasksQuery>,
) -> Result<impl IntoResponse> {
    match params.user_id {
        Some(user_id_str) => {
            let user_id = user_id_str
                .parse::<Uuid>()
                .map_err(|_| ApiError::bad_request(format!("Invalid user ID format: {}", user_id_str)))?;

            // Users can only view their own tasks, admins can view any user's tasks
            if current_user.role != UserRole::Admin && current_user.id != user_id {
                return Err(ApiError::forbidden("You can only view your own tasks"));
            }

            let tasks = task_service.get_tasks_by_user(user_id).await?;
            Ok(respond_ok(tasks))
        }
        None => {
            // Only admins can view all tasks
            if current_user.role != UserRole::Admin {
                return Err(ApiError::forbidden("Only administrators can view all tasks"));
            }

            let tasks = task_service.get_all_tasks().await?;
            Ok(respond_ok(tasks))
        }
    }
}