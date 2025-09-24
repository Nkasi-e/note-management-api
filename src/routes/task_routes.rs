use axum::{
    routing::{get, post},
    Router,
};

use crate::handlers::{create_task, get_task, get_tasks};
use crate::services::TaskService;

pub fn task_routes() -> Router<TaskService> {
    Router::new()
        .route("/tasks/", post(create_task))
        .route("/tasks/", get(get_tasks))
        .route("/tasks/:id", get(get_task))
}
