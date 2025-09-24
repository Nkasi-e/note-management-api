use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::domain::{Task, CreateTaskRequest, Result, ApiError};
use crate::services::TaskService;

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
    Json(request): Json<CreateTaskRequest>,
) -> Result<Json<Task>> {
    let task = task_service.create_task(request).await?;
    Ok(Json(task))
}

pub async fn get_task(
    State(task_service): State<TaskService>,
    Path(params): Path<TaskIdPath>,
) -> Result<Json<Task>> {
    let task_id = params
        .id
        .parse::<Uuid>()
        .map_err(|_| ApiError::InvalidUuid(params.id))?;

    let task = task_service.get_task(task_id).await?;
    Ok(Json(task))
}

pub async fn get_tasks(
    State(task_service): State<TaskService>,
    Query(params): Query<TasksQuery>,
) -> Result<Json<Vec<Task>>> {
    match params.user_id {
        Some(user_id_str) => {
            let user_id = user_id_str
                .parse::<Uuid>()
                .map_err(|_| ApiError::InvalidUuid(user_id_str))?;

            let tasks = task_service.get_tasks_by_user(user_id).await?;
            Ok(Json(tasks))
        }
        None => {
            let tasks = task_service.get_all_tasks().await?;
            Ok(Json(tasks))
        }
    }
}
