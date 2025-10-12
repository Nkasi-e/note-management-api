use axum::Router;
use std::sync::Arc;

use crate::services::{UserService, TaskService, AuthService, EmailService};
use crate::middleware::auth_middleware;
use crate::config::settings::AuthConfig;
use crate::workers::WorkerService;

use super::{user_routes, task_routes, auth_routes, worker_routes};

pub fn api_v1_routes(
    user_service: UserService,
    task_service: TaskService,
    auth_service: AuthService,
    auth_config: AuthConfig,
    worker_service: Arc<WorkerService>,
    email_service: Arc<EmailService>,
) -> Router {
    Router::new()
        .nest("/api/v1", Router::new()
            .nest("/auth", auth_routes()
                .with_state(auth_service)
                .layer(axum::Extension(worker_service.clone()))
                .layer(axum::Extension(email_service.clone()))
            )
            .nest("/users", 
                user_routes()
                    .with_state(user_service)
                    .layer(axum::middleware::from_fn_with_state(auth_config.clone(), auth_middleware))
            )
            .nest("/tasks", 
                task_routes()
                    .with_state(task_service)
                    .layer(axum::middleware::from_fn_with_state(auth_config.clone(), auth_middleware))
            )
            .nest("/workers", 
                worker_routes(worker_service, auth_config)
            )
        )
}
