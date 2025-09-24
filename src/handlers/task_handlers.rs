use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::domain::{Task, CreateTaskRequest, Result, ApiError};
use crate::services::TaskService;
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
    Json(request): Json<CreateTaskRequest>,
) -> Result<impl IntoResponse> {
    let task = task_service.create_task(request).await?;
    Ok(respond_created(task))
}

pub async fn get_task(
    State(task_service): State<TaskService>,
    Path(params): Path<TaskIdPath>,
) -> Result<impl IntoResponse> {
    let task_id = params
        .id
        .parse::<Uuid>()
        .map_err(|_| ApiError::InvalidUuid(params.id))?;

    let task = task_service.get_task(task_id).await?;
    Ok(respond_ok(task))
}

pub async fn get_tasks(
    State(task_service): State<TaskService>,
    Query(params): Query<TasksQuery>,
) -> Result<impl IntoResponse> {
    match params.user_id {
        Some(user_id_str) => {
            let user_id = user_id_str
                .parse::<Uuid>()
                .map_err(|_| ApiError::InvalidUuid(user_id_str))?;

            let tasks = task_service.get_tasks_by_user(user_id).await?;
            Ok(respond_ok(tasks))
        }
        None => {
            let tasks = task_service.get_all_tasks().await?;
            Ok(respond_ok(tasks))
        }
    }
}
