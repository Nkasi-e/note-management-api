use axum::{
    routing::{get, post},
    Router,
};

use crate::handlers::{create_task, get_task, get_tasks};
use crate::services::TaskService;

pub fn task_routes() -> Router<TaskService> {
    Router::new()
        .route("/", post(create_task))
        .route("/", get(get_tasks))
        .route("/:id", get(get_task))
}
