use axum::{
    extract::{Path, Query, State, Extension},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

/// Response wrapper for dynamic task queries
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum TaskQueryResponse {
    Simple(Vec<crate::domain::Task>),
    Paginated(PaginatedResponse<crate::domain::Task>),
}
use uuid::Uuid;
use tracing::{info, debug};

use crate::domain::{CreateTaskRequest, Result, ApiError, TaskQueryParams, PaginatedResponse};
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

/// Dynamic task query parameters - supports both simple and paginated queries
#[derive(Debug, Deserialize)]
pub struct DynamicTaskQuery {
    // Simple query parameters (for backward compatibility)
    pub user_id: Option<String>,
    
    // Pagination parameters (optional - if provided, enables pagination)
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub sort_by: Option<String>,
    pub sort_direction: Option<String>,
    
    // Filter parameters
    pub status: Option<String>,
    pub created_after: Option<String>,
    pub created_before: Option<String>,
    pub search: Option<String>,
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
    Query(params): Query<DynamicTaskQuery>,
) -> Result<impl IntoResponse> {
    info!("Getting tasks for user: {}", current_user.id);
    debug!("Query parameters: {:?}", params);

    // Check if pagination parameters are provided
    let has_pagination_params = params.page.is_some() || params.limit.is_some() || 
                              params.sort_by.is_some() || params.sort_direction.is_some() ||
                              params.status.is_some() || params.created_after.is_some() ||
                              params.created_before.is_some() || params.search.is_some();

    let response = if has_pagination_params {
        // Use paginated query
        let query_params = convert_to_task_query_params(params, &current_user)?;
        let result = task_service.get_tasks_paginated(query_params).await?;
        
        info!("Returning {} tasks on page {}/{}", 
              result.data.len(), 
              result.pagination.page, 
              result.pagination.total_pages);
        
        TaskQueryResponse::Paginated(result)
    } else {
        // Use simple query (backward compatibility)
        let tasks = match params.user_id {
            Some(user_id_str) => {
                let user_id = user_id_str
                    .parse::<Uuid>()
                    .map_err(|_| ApiError::bad_request(format!("Invalid user ID format: {}", user_id_str)))?;

                // Users can only view their own tasks, admins can view any user's tasks
                if current_user.role != UserRole::Admin && current_user.id != user_id {
                    return Err(ApiError::forbidden("You can only view your own tasks"));
                }

                task_service.get_tasks_by_user(user_id).await?
            }
            None => {
                // Only admins can view all tasks
                if current_user.role != UserRole::Admin {
                    return Err(ApiError::forbidden("Only administrators can view all tasks"));
                }

                task_service.get_all_tasks().await?
            }
        };
        
        TaskQueryResponse::Simple(tasks)
    };

    Ok(respond_ok(response))
}

/// Convert DynamicTaskQuery to TaskQueryParams
fn convert_to_task_query_params(
    params: DynamicTaskQuery, 
    current_user: &CurrentUser
) -> Result<TaskQueryParams> {
    use crate::domain::{PaginationParams, TaskFilters, TaskQueryParams};
    use crate::domain::task::TaskStatus;
    use chrono::{DateTime, Utc};

    // Build pagination params
    let pagination = PaginationParams {
        page: params.page.unwrap_or(1),
        limit: params.limit.unwrap_or(20),
        sort_by: params.sort_by.unwrap_or_else(|| "created_at".to_string()),
        sort_direction: params.sort_direction.unwrap_or_else(|| "desc".to_string()),
    };

    // Build filters
    let mut filters = TaskFilters::default();
    
    // Parse status if provided
    if let Some(status_str) = params.status {
        let status = match status_str.to_lowercase().as_str() {
            "todo" => TaskStatus::Todo,
            "in_progress" => TaskStatus::InProgress,
            "done" => TaskStatus::Done,
            _ => return Err(ApiError::bad_request("Invalid status. Must be: todo, in_progress, done")),
        };
        filters.status = Some(status);
    }

    // Parse user_id if provided
    if let Some(user_id_str) = params.user_id {
        let user_id = user_id_str
            .parse::<Uuid>()
            .map_err(|_| ApiError::bad_request("Invalid user ID format"))?;
        filters.user_id = Some(user_id);
    } else if current_user.role != crate::domain::user::UserRole::Admin {
        // Non-admin users can only see their own tasks
        filters.user_id = Some(current_user.id);
    }

    // Parse date filters if provided
    if let Some(created_after_str) = params.created_after {
        let created_after = created_after_str
            .parse::<DateTime<Utc>>()
            .map_err(|_| ApiError::bad_request("Invalid created_after date format. Use ISO 8601"))?;
        filters.created_after = Some(created_after);
    }

    if let Some(created_before_str) = params.created_before {
        let created_before = created_before_str
            .parse::<DateTime<Utc>>()
            .map_err(|_| ApiError::bad_request("Invalid created_before date format. Use ISO 8601"))?;
        filters.created_before = Some(created_before);
    }

    // Add search filter if provided
    if let Some(search) = params.search {
        if !search.trim().is_empty() {
            filters.search = Some(search.trim().to_string());
        }
    }

    Ok(TaskQueryParams {
        pagination,
        filters,
    })
}